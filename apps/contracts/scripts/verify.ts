import { run } from "hardhat";
import * as fs from "fs";
import * as path from "path";

async function main() {
  console.log("🔍 Verifying PaymentEscrow contract on BSCScan...\n");

  // Read deployment info
  const networkName = process.env.HARDHAT_NETWORK === "bscMainnet" ? "mainnet" : "testnet";
  const deploymentFile = path.join(__dirname, "../deployments", `${networkName}.json`);

  if (!fs.existsSync(deploymentFile)) {
    throw new Error(`Deployment file not found: ${deploymentFile}`);
  }

  const deploymentInfo = JSON.parse(fs.readFileSync(deploymentFile, "utf8"));
  const { contractAddress, adminWallet } = deploymentInfo;

  console.log("📝 Contract Address:", contractAddress);
  console.log("👤 Admin Wallet:", adminWallet, "\n");

  try {
    await run("verify:verify", {
      address: contractAddress,
      constructorArguments: [adminWallet],
    });

    console.log("\n✅ Contract verified successfully on BSCScan!");
  } catch (error: any) {
    if (error.message.includes("Already Verified")) {
      console.log("\n✅ Contract already verified on BSCScan!");
    } else {
      console.error("\n❌ Verification failed:", error.message);
      throw error;
    }
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ Error:", error);
    process.exit(1);
  });
