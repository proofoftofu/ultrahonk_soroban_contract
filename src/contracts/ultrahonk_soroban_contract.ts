import * as Client from 'ultrahonk_soroban_contract';
import { rpcUrl } from './util';

export default new Client.Client({
  networkPassphrase: 'Standalone Network ; February 2017',
  contractId: 'CCYFHQLAPB7CHBBE7QIN2QEBEBJPSGRJ2OJ4JPCHIN5IPKTVQ7YCR2CI',
  rpcUrl,
  allowHttp: true,
  publicKey: undefined,
});
