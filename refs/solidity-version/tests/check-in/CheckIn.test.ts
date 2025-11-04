import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import { expect } from "chai";
import { Contract } from "ethers";
import { ethers } from "hardhat";

describe("CheckIn", function () {
    let checkIn: Contract;
    let minimalForwarder: Contract;
    let deployer: SignerWithAddress;
    let user: SignerWithAddress;
    let forwarder: SignerWithAddress;

    beforeEach(async function () {
        [deployer, user, forwarder] = await ethers.getSigners();
        const minimalForwarderFactory = await ethers.getContractFactory("Forwarder");
        minimalForwarder = await minimalForwarderFactory.deploy();
        await minimalForwarder.waitForDeployment();
        const checkInFactory = await ethers.getContractFactory("CheckIn");
        checkIn = await checkInFactory.deploy(await minimalForwarder.getAddress());
        await checkIn.waitForDeployment();
    });

    describe("Direct EOA check-in", function () {
        it("should allow user to check in", async function () {
            await checkIn.connect(user).userCheckIn();
            const checkInInfo = await checkIn.checkInInfos(user.address);
            expect(checkInInfo.count).to.equal(1);
            expect(checkInInfo.lastTime).to.be.gt(0);
        });

        it("should check-in twice in different block", async function () {
            await checkIn.connect(user).userCheckIn();
            const firstCheckInInfo = await checkIn.checkInInfos(user.address);
            console.log("First check-in lastTime:", firstCheckInInfo.lastTime.toString());
            await checkIn.connect(user).userCheckIn();
            const secondCheckInInfo = await checkIn.checkInInfos(user.address);
            console.log("Second check-in lastTime:", secondCheckInInfo.lastTime.toString());
            expect(secondCheckInInfo.lastTime).to.be.gt(firstCheckInInfo.lastTime);
        });
    });

    describe("ERC2771 meta-transaction check-in", function () {
        it("should allow check-in via meta-transaction", async function () {
            const domain = {
                name: "GSNv2 Forwarder",
                version: "0.0.1",
                chainId: (await ethers.provider.getNetwork()).chainId,
                verifyingContract: await minimalForwarder.getAddress(),
            };
            const types = {
                ForwardRequest: [
                    { name: "from", type: "address" },
                    { name: "to", type: "address" },
                    { name: "value", type: "uint256" },
                    { name: "gas", type: "uint256" },
                    { name: "nonce", type: "uint256" },
                    { name: "data", type: "bytes" },
                ],
            };
            const checkInData = checkIn.interface.encodeFunctionData("userCheckIn");
            const request = {
                from: user.address,
                to: await checkIn.getAddress(),
                value: 0,
                gas: 100000,
                nonce: await minimalForwarder.getNonce(user.address),
                data: checkInData,
            };
            const signature = await user.signTypedData(domain, types, request);
            await minimalForwarder.connect(forwarder).execute(request, signature);
            const checkInInfo = await checkIn.checkInInfos(user.address);
            expect(checkInInfo.count).to.equal(1);
            expect(checkInInfo.lastTime).to.be.gt(0);
        });
    });
});
