import { ethers, network } from "hardhat";
import * as fs from "fs";
import * as path from "path";

async function main() {
    const addressPath = path.join(__dirname, "../../contracts/swap-handler/deployedAddress.json");
    const addresses = JSON.parse(fs.readFileSync(addressPath, "utf8"));
    const networkName = network.name;
    const networkAddresses = addresses[networkName];
    if (!networkAddresses) {
        throw new Error(`No addresses found for network ${networkName}`);
    }
    const swapHandlerAddress = networkAddresses["SwapHandlerV2"];
    if (!swapHandlerAddress) {
        throw new Error(`SwapHandlerV2 address not found for network ${networkName}`);
    }

    const [user] = await ethers.getSigners();
    const swapHandlerFactory = await ethers.getContractFactory("SwapHandlerV2");
    const swapHandler = swapHandlerFactory.attach(swapHandlerAddress);
    console.log(`\nTesting on network: ${networkName}`);
    console.log("SwapHandlerV2 address:", swapHandlerAddress);
    console.log("User address:", user.address);

    // console.log("\nContract Status:");
    // console.log("Current Router:", await swapHandler.ROUTER());
    // console.log("Current WETH:", await swapHandler.WETH());

    // Base Mainnet
    // const WETH = "0x4200000000000000000000000000000000000006";
    // const USDC = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";

    // BSC Mainnet
    const WETH = "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c";
    const USDC = "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d";

    const ethAmount = ethers.parseEther("0.0001");
    const deadline = Math.floor(Date.now() / 1000) + 3600;
    try {
        const tx1 = await swapHandler.wrapSwapExactETHForTokens(
            0,
            [WETH, USDC],
            user.address,
            deadline,
            "Swap ETH to USDC",
            { value: ethAmount },
        );
        await tx1.wait();
        console.log("ETH to USDC swap successful!");
    } catch (error) {
        console.error("ETH to USDC swap failed:", error);
    }

    const usdcContract = await ethers.getContractAt("IERC20", USDC);
    const usdcAmount = BigInt(100000);
    try {
        console.log("Approving USDC...");
        const approveTx = await usdcContract.approve(swapHandlerAddress, usdcAmount);
        await approveTx.wait();
        console.log("USDC approved");
        const tx2 = await swapHandler.wrapSwapExactTokensForETH(
            usdcAmount,
            0,
            [USDC, WETH],
            user.address,
            deadline,
            "Swap USDC to ETH",
        );
        await tx2.wait();
        console.log("USDC to ETH swap successful!");
    } catch (error) {
        console.error("USDC to ETH swap failed:", error);
    }

    try {
        console.log("Approving USDC for token swap...");
        const approveTx = await usdcContract.approve(swapHandlerAddress, usdcAmount);
        await approveTx.wait();
        console.log("USDC approved");
        const tx3 = await swapHandler.wrapSwapExactTokensForTokens(
            usdcAmount,
            0,
            [USDC, WETH],
            user.address,
            deadline,
            "Swap USDC to WETH",
        );
        await tx3.wait();
        console.log("USDC to WETH swap successful!");
    } catch (error) {
        console.error("USDC to WETH swap failed:", error);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
