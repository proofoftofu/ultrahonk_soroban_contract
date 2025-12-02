import * as Client from 'guess_the_puzzle';
import { rpcUrl } from './util';

export default new Client.Client({
  networkPassphrase: 'Standalone Network ; February 2017',
  contractId: 'CCOBCE3MIRXNKA7AMWT2Y5R6IU6A734MM6OM67X7QQHBZTC4NP7D2SJT',
  rpcUrl,
  allowHttp: true,
  publicKey: undefined,
});
