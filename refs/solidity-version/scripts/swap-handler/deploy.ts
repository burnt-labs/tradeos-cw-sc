import { ethers, network } from "hardhat";
import * as fs from "fs";
import * as path from "path";

async function main() {
    const addressPath = path.join(__dirname, "../../contracts/swap-handler/deployedAddress.json");
    const addresses = JSON.parse(fs.readFileSync(addressPath, "utf8"));
    const networkName = network.name;
    if (!addresses[networkName]) {
        addresses[networkName] = {};
    }

    const [owner] = await ethers.getSigners();

    // Base Mainnet
    // const routerAddress = "0x4752ba5DBc23f44D87826276BF6Fd6b1C372aD24";

    // BSC Mainnet
    const routerAddress = "0x10ED43C718714eb63d5aA57B78B54704E256024E";

    const swapHandlerFactory = await ethers.getContractFactory("SwapHandlerV2");
    const swapHandler = await swapHandlerFactory.deploy(owner.address, routerAddress);
    await swapHandler.waitForDeployment();
    const swapHandlerAddress = await swapHandler.getAddress();
    console.log(`\nDeployment Info for network ${networkName}:`);
    console.log("SwapHandlerV2 deployed to:", swapHandlerAddress);
    console.log("Owner:", owner.address);
    console.log("Router:", routerAddress);

    addresses[networkName]["SwapHandlerV2"] = swapHandlerAddress;
    addresses[networkName]["Router"] = routerAddress;
    fs.writeFileSync(addressPath, JSON.stringify(addresses, null, 2));
    console.log(`\ndeployedAddress.json updated for network ${networkName}`);
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
