import * as Client from 'guess_the_puzzle';
import { getGuessThePuzzleContractId, getNetworkUrls, getNetworkPassphrase } from './util';

// Factory function that creates a new client instance with current config
// This ensures the contract ID, RPC URL, and network passphrase are always up-to-date when network/storage changes
export const createGuessThePuzzleClient = () => {
  const networkUrls = getNetworkUrls();
  const passphrase = getNetworkPassphrase();
  
  return new Client.Client({
    networkPassphrase: passphrase,
    contractId: getGuessThePuzzleContractId(),
    rpcUrl: networkUrls.rpcUrl,
    allowHttp: true,
    publicKey: undefined,
  });
};

// Export a default client instance for backward compatibility
// Note: This will use the contract ID from when the module was first loaded
// For dynamic contract IDs, use createGuessThePuzzleClient() instead
export default createGuessThePuzzleClient();
