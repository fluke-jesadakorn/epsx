import { ethers } from "hardhat";
import * as fs from "fs";
import * as path from "path";

async function main() {
  console.log("🚀 Deploying PaymentEscrow contract...\n");

  // Get deployer account
  const [deployer] = await ethers.getSigners();
  console.log("📝 Deploying with account:", deployer.address);
  console.log("💰 Account balance:", ethers.formatEther(await ethers.provider.getBalance(deployer.address)), "BNB\n");

  // Get admin wallet address from environment
  const adminWallet = process.env.ADMIN_WALLET_ADDRESS || deployer.address;
  console.log("👤 Admin wallet:", adminWallet, "\n");

  // Deploy PaymentEscrow contract
  console.log("⏳ Deploying PaymentEscrow...");
  const PaymentEscrow = await ethers.getContractFactory("PaymentEscrow");
  const paymentEscrow = await PaymentEscrow.deploy(adminWallet);
  await paymentEscrow.waitForDeployment();

  const contractAddress = await paymentEscrow.getAddress();
  console.log("✅ PaymentEscrow deployed to:", contractAddress, "\n");

  // Configure token addresses based on network
  const network = await ethers.provider.getNetwork();
  let USDT_ADDRESS: string;
  let USDC_ADDRESS: string;

  if (network.chainId === 56n) {
    // BSC Mainnet
    USDT_ADDRESS = "0x55d398326f99059fF775485246999027B3197955";
    USDC_ADDRESS = "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d";
    console.log("🌐 Network: BSC Mainnet (56)");
  } else if (network.chainId === 97n) {
    // BSC Testnet
    USDT_ADDRESS = "0x337610d27c682E347C9cD60BD4b3b107C9d34dDD";
    USDC_ADDRESS = "0x64544969ed7EBf5f083679233325356EbE738930";
    console.log("🌐 Network: BSC Testnet (97)");
  } else {
    throw new Error(`Unsupported network: ${network.chainId}`);
  }

  console.log("🪙 USDT Address:", USDT_ADDRESS);
  console.log("🪙 USDC Address:", USDC_ADDRESS, "\n");

  // Configure tokens
  console.log("⏳ Enabling tokens...");
  await paymentEscrow.setTokenEnabled(USDT_ADDRESS, true);
  console.log("✅ USDT enabled");
  await paymentEscrow.setTokenEnabled(USDC_ADDRESS, true);
  console.log("✅ USDC enabled\n");

  // Configure plan prices (6 decimals for USDT/USDC)
  console.log("⏳ Setting plan prices...");
  const PLAN_STARTER = 1;
  const PLAN_PROFESSIONAL = 2;
  const PLAN_ENTERPRISE = 3;

  const PRICE_STARTER = ethers.parseUnits("29", 6); // $29
  const PRICE_PROFESSIONAL = ethers.parseUnits("59", 6); // $59
  const PRICE_ENTERPRISE = ethers.parseUnits("99", 6); // $99

  await paymentEscrow.setPlanPrice(PLAN_STARTER, PRICE_STARTER);
  console.log("✅ Plan 1 (Starter): $29");
  await paymentEscrow.setPlanPrice(PLAN_PROFESSIONAL, PRICE_PROFESSIONAL);
  console.log("✅ Plan 2 (Professional): $59");
  await paymentEscrow.setPlanPrice(PLAN_ENTERPRISE, PRICE_ENTERPRISE);
  console.log("✅ Plan 3 (Enterprise): $99\n");

  // Save deployment information
  const deploymentInfo = {
    network: network.name,
    chainId: network.chainId.toString(),
    contractAddress: contractAddress,
    adminWallet: adminWallet,
    tokens: {
      USDT: USDT_ADDRESS,
      USDC: USDC_ADDRESS,
    },
    plans: {
      1: { name: "Starter", price: "29" },
      2: { name: "Professional", price: "59" },
      3: { name: "Enterprise", price: "99" },
    },
    deployedAt: new Date().toISOString(),
    deployer: deployer.address,
  };

  const deploymentsDir = path.join(__dirname, "../deployments");
  if (!fs.existsSync(deploymentsDir)) {
    fs.mkdirSync(deploymentsDir, { recursive: true });
  }

  const networkName = network.chainId === 56n ? "mainnet" : "testnet";
  const deploymentFile = path.join(deploymentsDir, `${networkName}.json`);
  fs.writeFileSync(deploymentFile, JSON.stringify(deploymentInfo, null, 2));

  console.log("📄 Deployment info saved to:", deploymentFile, "\n");

  // Export ABI
  const artifact = await ethers.getContractFactory("PaymentEscrow").then(f => f.interface);
  const abiFile = path.join(deploymentsDir, "PaymentEscrow.json");
  fs.writeFileSync(
    abiFile,
    JSON.stringify({
      contractName: "PaymentEscrow",
      abi: artifact.formatJson(),
    }, null, 2)
  );
  console.log("📄 ABI exported to:", abiFile, "\n");

  console.log("🎉 Deployment complete!\n");
  console.log("📋 Summary:");
  console.log("Contract Address:", contractAddress);
  console.log("Admin Wallet:", adminWallet);
  console.log("USDT:", USDT_ADDRESS);
  console.log("USDC:", USDC_ADDRESS);
  console.log("\n🔍 Verify contract on BSCScan:");
  console.log(`npx hardhat verify --network ${networkName === "mainnet" ? "bscMainnet" : "bscTestnet"} ${contractAddress} "${adminWallet}"`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ Error:", error);
    process.exit(1);
  });
