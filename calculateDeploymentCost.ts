import { execSync } from 'child_process';
import fs from 'fs';
import { Connection, LAMPORTS_PER_SOL, clusterApiUrl, PublicKey } from '@solana/web3.js';
import { PythHttpClient, getPythProgramKeyForCluster } from '@pythnetwork/client';

const RENT_PER_BYTE_YEAR = 0.00000348 * LAMPORTS_PER_SOL;
const BUFFER_ACCOUNT_COST = 0.1 * LAMPORTS_PER_SOL; // Estimated
const TRANSACTION_FEE = 0.000005 * LAMPORTS_PER_SOL;

async function calculateDeploymentCost() {
    // Build the program
    console.log('Building the program...');
    execSync('anchor build', { stdio: 'inherit' });

    // Get the program binary path
    const programPath = 'target/deploy/blink_take_2.so';

    // Get the size of the program binary
    const stats = fs.statSync(programPath);
    const programSize = stats.size;
    console.log(`Program size: ${programSize} bytes`);

    // Calculate rent exemption
    const rentExemption = (programSize * RENT_PER_BYTE_YEAR) / LAMPORTS_PER_SOL;
    console.log(`Rent exemption: ${rentExemption.toFixed(6)} SOL`);

    // Calculate total deployment cost
    const totalCost = rentExemption + (BUFFER_ACCOUNT_COST / LAMPORTS_PER_SOL) + (TRANSACTION_FEE / LAMPORTS_PER_SOL);
    console.log(`Estimated total deployment cost: ${totalCost.toFixed(6)} SOL`);

    // Get current SOL price
    try {
        const connection = new Connection(clusterApiUrl('mainnet-beta'));
        const pythClient = new PythHttpClient(connection, getPythProgramKeyForCluster('mainnet-beta'));
        
        const solUsdPriceAccount = new PublicKey('H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG');
        const data = await pythClient.getAssetPricesFromAccounts([solUsdPriceAccount]);
        const solUsdPrice = data[0];
        
        if (solUsdPrice && solUsdPrice.price) {
            const costInUSD = totalCost * solUsdPrice.price;
            console.log(`Estimated cost in USD: $${costInUSD.toFixed(2)}`);
        } else {
            console.log('Unable to fetch current SOL price or price is not available.');
        }
    } catch (error) {
        console.error('Failed to fetch SOL price:', error);
    }
}

calculateDeploymentCost().catch(console.error);