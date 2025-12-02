import { useState } from "react";
import { Button, Modal } from "@stellar/design-system";
import storage from "../util/storage";
import { stellarNetwork } from "../contracts/util";

export type NetworkOption = {
  id: string;
  label: string;
  rpcUrl: string;
  horizonUrl: string;
  passphrase: string;
};

const NETWORKS: NetworkOption[] = [
  {
    id: "LOCAL",
    label: "Local",
    rpcUrl: "http://localhost:8000/rpc",
    horizonUrl: "http://localhost:8000",
    passphrase: "Standalone Network ; February 2017",
  },
  {
    id: "TESTNET",
    label: "Testnet",
    rpcUrl: "https://soroban-testnet.stellar.org:443",
    horizonUrl: "https://horizon-testnet.stellar.org",
    passphrase: "Test SDF Network ; September 2015",
  },
  {
    id: "FUTURENET",
    label: "Futurenet",
    rpcUrl: "https://rpc-futurenet.stellar.org:443",
    horizonUrl: "https://horizon-futurenet.stellar.org",
    passphrase: "Test SDF Future Network ; October 2022",
  },
  {
    id: "PUBLIC",
    label: "Mainnet",
    rpcUrl: "https://soroban-rpc.mainnet.stellar.org:443",
    horizonUrl: "https://horizon.stellar.org",
    passphrase: "Public Global Stellar Network ; September 2015",
  },
  {
    id: "NOIR",
    label: "NOIR",
    rpcUrl: "https://noir-local.stellar.buzz/soroban/rpc",
    horizonUrl: "https://noir-local.stellar.buzz",
    passphrase: "Standalone Network ; February 2017",
  },
];

export const NetworkSwitcher: React.FC = () => {
  const [showModal, setShowModal] = useState(false);
  const currentNetwork = stellarNetwork;

  const handleNetworkSelect = (network: NetworkOption) => {
    storage.setItem("selectedNetwork", network.id);
    setShowModal(false);
    // Reload the page to apply network changes
    window.location.reload();
  };

  return (
    <>
      <Button
        variant="tertiary"
        size="md"
        onClick={() => setShowModal(true)}
      >
        Network
      </Button>
      <div id="networkSwitcherModalContainer">
        <Modal
          visible={showModal}
          onClose={() => setShowModal(false)}
          parentId="networkSwitcherModalContainer"
        >
          <Modal.Heading>Select Network</Modal.Heading>
          <Modal.Body>
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "8px",
              }}
            >
              {NETWORKS.map((network) => (
                <Button
                  key={network.id}
                  variant={currentNetwork === network.id ? "primary" : "secondary"}
                  size="md"
                  onClick={() => handleNetworkSelect(network)}
                  style={{
                    width: "100%",
                    justifyContent: "flex-start",
                  }}
                >
                  {network.label}
                </Button>
              ))}
            </div>
          </Modal.Body>
          <Modal.Footer>
            <Button
              size="md"
              variant="tertiary"
              onClick={() => setShowModal(false)}
            >
              Cancel
            </Button>
          </Modal.Footer>
        </Modal>
      </div>
    </>
  );
};

export { NetworkSwitcher as default };

