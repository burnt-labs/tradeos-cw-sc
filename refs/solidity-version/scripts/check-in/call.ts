import { ethers, network } from "hardhat";
import * as fs from "fs";
import * as path from "path";

async function printStorageVariables(contract, mockUserAddress) {
    const totalCount = await contract.checkInTotalCount();
    console.log(`checkInTotalCount: ${totalCount.toString()}`);
    const checkInInfo = await contract.checkInInfos(mockUserAddress);
    console.log(`checkInInfo for ${mockUserAddress}:`);
    console.log(`  Count: ${checkInInfo.count.toString()}`);
    console.log(`  Last Time: ${checkInInfo.lastTime.toString()}`);
}

async function main() {
    const addressPath = path.join(__dirname, "../../contracts/check-in/deployedAddress.json");
    const addresses = JSON.parse(fs.readFileSync(addressPath, "utf8"));
    const networkName = network.name;
    const networkAddresses = addresses[networkName];
    if (!networkAddresses) {
        throw new Error(`No addresses found for network ${networkName}`);
    }

    const trustedForwarderAddress = networkAddresses["MinimalForwarder"];
    const checkInAddress = networkAddresses["CheckIn"];
    if (!checkInAddress || !trustedForwarderAddress) {
        throw new Error(`Required addresses not found for network ${networkName}`);
    }
    const minimalForwarderFactory = await ethers.getContractFactory("Forwarder");
    const minimalForwarder = minimalForwarderFactory.attach(trustedForwarderAddress);
    const checkInFactory = await ethers.getContractFactory("CheckIn");
    const checkInContract = checkInFactory.attach(checkInAddress);

    const [relayer, mockUser] = await ethers.getSigners();
    const relayerAddress = await relayer.getAddress();
    const mockUserAddress = await mockUser.getAddress();

    console.log(`\nTesting on network: ${networkName}`);
    console.log("Attempting to check in via EOA...");
    let tx = await checkInContract.connect(relayer).userCheckIn();
    await tx.wait();
    console.log("Checked in successfully via EOA");
    await printStorageVariables(checkInContract, relayerAddress);
    await new Promise((resolve) => setTimeout(resolve, 12000));

    console.log("\nAttempting to check in via ERC2771 meta-transaction...");
    const domain = {
        name: "GSNv2 Forwarder",
        version: "0.0.1",
        chainId: (await ethers.provider.getNetwork()).chainId,
        verifyingContract: trustedForwarderAddress,
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
    const checkInData = checkInContract.interface.encodeFunctionData("userCheckIn");
    const request = {
        from: mockUserAddress,
        to: checkInAddress,
        value: 0,
        gas: 100000,
        nonce: await minimalForwarder.getNonce(mockUserAddress),
        data: checkInData,
    };
    const signature = await mockUser.signTypedData(domain, types, request);
    tx = await minimalForwarder.connect(relayer).execute(request, signature);
    await tx.wait();
    console.log("Checked in successfully via ERC2771 meta-transaction");
    await printStorageVariables(checkInContract, mockUserAddress);
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
