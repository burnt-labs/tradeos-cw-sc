import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import { time } from "@nomicfoundation/hardhat-network-helpers";
import { expect } from "chai";
import { ethers } from "hardhat";
import { Vault, Points } from "../../typechain-types";

describe("Vault", () => {
    let points: Points;
    let pointsAddress: string;
    let vault: Vault;
    let vaultAddress: string;
    let owner: SignerWithAddress;
    let verifier: SignerWithAddress;
    let user: SignerWithAddress;
    let otherUser: SignerWithAddress;

    beforeEach(async () => {
        [owner, verifier, user, otherUser] = await ethers.getSigners();

        const vaultFactory = await ethers.getContractFactory("Vault");
        vault = (await vaultFactory.deploy(owner.address, verifier.address)) as Vault;
        await vault.waitForDeployment();
        const pointsFactory = await ethers.getContractFactory("Points");
        points = (await pointsFactory.deploy(owner.address)) as Points;
        await points.waitForDeployment();

        vaultAddress = await vault.getAddress();
        pointsAddress = await points.getAddress();
        await points.mint(vaultAddress, ethers.parseEther("10000"));
    });

    describe("claim", () => {
        const value = ethers.parseEther("100");
        let deadline: number;
        let signature: string;
        let claimInfo: any;

        beforeEach(async () => {
            deadline = (await time.latest()) + 60;
            claimInfo = {
                token: pointsAddress,
                to: user.address,
                value: value,
                deadline: deadline,
                comment: "Test Token Claim",
            };
            const messageHash = await vault.getClaimInfoHash(claimInfo);
            signature = await verifier.signMessage(ethers.getBytes(messageHash));
        });

        describe("validation checks", () => {
            it("should revert with InvalidValue for zero or insufficient balance claims", async () => {
                claimInfo.value = 0;
                let messageHash = await vault.getClaimInfoHash(claimInfo);
                let newSignature = await verifier.signMessage(ethers.getBytes(messageHash));
                await expect(vault.connect(user).claim(claimInfo, newSignature)).to.be.rejectedWith("InvalidValue");

                const contractBalance = await points.balanceOf(vaultAddress);
                claimInfo.value = contractBalance + 1n;
                messageHash = await vault.getClaimInfoHash(claimInfo);
                newSignature = await verifier.signMessage(ethers.getBytes(messageHash));
                await expect(vault.connect(user).claim(claimInfo, newSignature)).to.be.rejectedWith(
                    "InsufficientBalance",
                );

                await vault.connect(owner).emergencyWithdraw(pointsAddress, owner.address, contractBalance);
                await expect(vault.connect(user).claim(claimInfo, newSignature)).to.be.rejectedWith(
                    "InsufficientBalance",
                );
            });

            it("should revert with SignatureExpired for expired claims", async () => {
                await time.increase(3601);
                await expect(vault.connect(user).claim(claimInfo, signature)).to.be.rejectedWith("SignatureExpired");
            });

            it("should revert with AlreadyClaimed for reused signatures", async () => {
                await vault.connect(user).claim(claimInfo, signature);
                await expect(vault.connect(user).claim(claimInfo, signature)).to.be.rejectedWith("AlreadyClaimed");
            });

            it("should revert with InvalidSignature for invalid signer", async () => {
                const fakeSignature = await otherUser.signMessage(
                    ethers.getBytes(await vault.getClaimInfoHash(claimInfo)),
                );
                await expect(vault.connect(user).claim(claimInfo, fakeSignature)).to.be.rejectedWith(
                    "InvalidSignature",
                );
            });

            it("should handle empty comment string", async () => {
                claimInfo.comment = "";
                const messageHash = await vault.getClaimInfoHash(claimInfo);
                const newSignature = await verifier.signMessage(ethers.getBytes(messageHash));
                await expect(vault.connect(user).claim(claimInfo, newSignature)).to.not.be.rejected;
            });
        });

        describe("successful claims", () => {
            it("should transfer tokens and emit event on successful claim", async () => {
                const balanceBefore = await points.balanceOf(user.address);
                const tx = await vault.connect(user).claim(claimInfo, signature);
                await tx.wait();

                const receipt = await tx.wait();
                const event = receipt?.logs[receipt.logs.length - 1];
                expect(event?.eventName).to.equal("Claimed");

                const args = event?.args;
                expect(args?.token).to.equal(pointsAddress);
                expect(args?.to).to.equal(user.address);
                expect(args?.value).to.equal(value);
                expect(args?.comment).to.equal("Test Token Claim");

                const balanceAfter = await points.balanceOf(user.address);
                expect(balanceAfter - balanceBefore).to.equal(value);
            });

            it("should transfer ETH and emit event on successful ETH claim", async () => {
                const ethValue = ethers.parseEther("1.0");
                await owner.sendTransaction({
                    to: vaultAddress,
                    value: ethValue,
                });
                const ethClaimInfo = {
                    token: ethers.ZeroAddress,
                    to: user.address,
                    value: ethValue,
                    deadline: deadline,
                    comment: "Test ETH Claim",
                };
                const messageHash = await vault.getClaimInfoHash(ethClaimInfo);
                const ethSignature = await verifier.signMessage(ethers.getBytes(messageHash));

                const balanceBefore = await ethers.provider.getBalance(user.address);
                const tx = await vault.connect(user).claim(ethClaimInfo, ethSignature);
                const receipt = await tx.wait();

                const event = receipt?.logs[receipt.logs.length - 1];
                expect(event?.eventName).to.equal("Claimed");

                const args = event?.args;
                expect(args?.token).to.equal(ethers.ZeroAddress);
                expect(args?.to).to.equal(user.address);
                expect(args?.value).to.equal(ethValue);
                expect(args?.comment).to.equal("Test ETH Claim");

                const balanceAfter = await ethers.provider.getBalance(user.address);
                const gasCost = receipt.gasUsed * receipt.gasPrice;
                expect(balanceAfter + gasCost - balanceBefore).to.equal(ethValue);
            });
        });
    });

    describe("setVerifier", () => {
        it("should update verifier and emit event when called by owner", async () => {
            const tx = await vault.connect(owner).setVerifier(otherUser.address);
            await tx.wait();
            const receipt = await tx.wait();
            const event = receipt?.logs[receipt.logs.length - 1];
            expect(event?.eventName).to.equal("VerifierUpdated");
            expect(await vault.verifier()).to.equal(otherUser.address);
        });

        it("should revert when called by non-owner", async () => {
            await expect(vault.connect(user).setVerifier(otherUser.address)).to.be.rejectedWith(
                "Ownable: caller is not the owner",
            );
        });

        it("should revert with InvalidAddress for zero address", async () => {
            await expect(vault.connect(owner).setVerifier(ethers.ZeroAddress)).to.be.rejectedWith("InvalidAddress");
        });
    });

    describe("emergencyWithdraw", () => {
        it("should allow owner to withdraw tokens and update balances", async () => {
            const amount = ethers.parseEther("10000");
            const balanceBefore = await points.balanceOf(otherUser.address);
            const contractBalanceBefore = await points.balanceOf(vaultAddress);
            await vault.connect(owner).emergencyWithdraw(pointsAddress, otherUser.address, amount);
            const balanceAfter = await points.balanceOf(otherUser.address);
            const contractBalanceAfter = await points.balanceOf(vaultAddress);
            expect(balanceAfter - balanceBefore).to.equal(amount);
            expect(contractBalanceBefore - contractBalanceAfter).to.equal(amount);
        });

        it("should revert when called by non-owner", async () => {
            await expect(
                vault.connect(user).emergencyWithdraw(pointsAddress, user.address, ethers.parseEther("10000")),
            ).to.be.revertedWith("Ownable: caller is not the owner");
        });
    });

    describe("Points integration", () => {
        it("should allow owner to mint new tokens", async () => {
            const mintAmount = ethers.parseEther("10000");
            const balanceBefore = await points.balanceOf(user.address);
            await points.mint(user.address, mintAmount);
            const balanceAfter = await points.balanceOf(user.address);
            expect(balanceAfter - balanceBefore).to.equal(mintAmount);
        });

        it("should allow owner to burn tokens", async () => {
            const mintAmount = ethers.parseEther("10000");
            const burnAmount = ethers.parseEther("5000");
            await points.mint(user.address, mintAmount);
            const balanceAfterMint = await points.balanceOf(user.address);
            await points.burn(user.address, burnAmount);
            const balanceAfterBurn = await points.balanceOf(user.address);
            expect(balanceAfterMint - balanceAfterBurn).to.equal(burnAmount);
        });

        it("should revert when non-owner tries to mint", async () => {
            await expect(points.connect(user).mint(user.address, ethers.parseEther("10000"))).to.be.revertedWith(
                "Ownable: caller is not the owner",
            );
        });

        it("should revert when non-owner tries to burn", async () => {
            await expect(points.connect(user).burn(user.address, ethers.parseEther("10000"))).to.be.revertedWith(
                "Ownable: caller is not the owner",
            );
        });
    });
});
