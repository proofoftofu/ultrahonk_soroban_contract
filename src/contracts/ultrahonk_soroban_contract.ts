import * as Client from 'ultrahonk_soroban_contract';
import { rpcUrl, networkPassphrase, getUltrahonkContractId } from './util';

export default new Client.Client({
  networkPassphrase,
  contractId: getUltrahonkContractId(),
  rpcUrl,
  allowHttp: true,
  publicKey: undefined,
});
