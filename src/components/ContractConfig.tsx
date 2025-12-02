import { useState, useEffect } from "react";
import { Button, Input, Text } from "@stellar/design-system";
import { Box } from "./layout/Box";
import storage from "../util/storage";
import { stellarNetwork, getGuessThePuzzleContractId } from "../contracts/util";

export const ContractConfig = () => {
  const [contractId, setContractId] = useState<string>("");
  const [isUpdating, setIsUpdating] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);
  const [hasManualOverride, setHasManualOverride] = useState(false);

  // Load contract ID on mount and when network changes
  useEffect(() => {
    const stored = storage.getItem('contractId', 'safe');
    const envDefault = getGuessThePuzzleContractId(true);
    
    // When network changes, always update input field to env default
    // If user had an override that differs from the env default, we'll keep showing it
    if (stored && stored !== envDefault) {
      // User has a manual override that differs from env default
      setContractId(stored);
      setHasManualOverride(true);
    } else {
      // Use env default
      setContractId(envDefault);
      setHasManualOverride(false);
      // Clear storage to use env default
      if (stored) {
        storage.removeItem('contractId');
      }
    }
  }, [stellarNetwork]);

  const handleUpdate = () => {
    if (!contractId.trim()) {
      setMessage({ type: "error", text: "Contract ID cannot be empty" });
      return;
    }

    // Basic validation: Stellar contract IDs are 56 characters
    if (contractId.length !== 56) {
      setMessage({ type: "error", text: "Invalid contract ID format (must be 56 characters)" });
      return;
    }

    setIsUpdating(true);
    setMessage(null);

    try {
      // Store the contract ID as a manual override
      storage.setItem('contractId', contractId);
      setHasManualOverride(true);
      setMessage({ type: "success", text: "Contract address updated successfully! Please refresh the page to use the new contract." });
      
      // Clear message after 5 seconds
      setTimeout(() => {
        setMessage(null);
      }, 5000);
    } catch (error: any) {
      setMessage({ type: "error", text: `Failed to update: ${error.message}` });
    } finally {
      setIsUpdating(false);
    }
  };

  const handleInputChange = (value: string) => {
    setContractId(value);
    setMessage(null);
    // If user clears the field or changes it, mark as potential manual override
    // (will be confirmed on Update button click)
  };

  const handleResetToDefault = () => {
    const defaultId = getGuessThePuzzleContractId(true);
    setContractId(defaultId);
    storage.removeItem('contractId');
    setHasManualOverride(false);
    setMessage({ type: "success", text: "Reset to environment default. Please refresh the page." });
    setTimeout(() => {
      setMessage(null);
    }, 5000);
  };

  return (
    <Box gap="xs" direction="column">
      <Box gap="xs" direction="row" align="baseline" wrap="wrap">
        <Text as="h2" size="md" style={{ margin: 0 }}>
          Contract Address
        </Text>
        <Text as="p" size="xs" style={{ margin: 0, color: "#6b7280" }}>
          (56 chars)
        </Text>
      </Box>
      
      <Box gap="sm" direction="row" align="end" wrap="nowrap">
        <Input
          label=""
          id="contract-id"
          fieldSize="md"
          value={contractId}
          onChange={(e) => handleInputChange(e.target.value)}
          placeholder="Enter contract ID"
          style={{ width: "500px", fontFamily: "monospace", fontSize: "0.85rem", flexShrink: 0 }}
        />
        <Button
          onClick={handleUpdate}
          disabled={isUpdating || !contractId.trim()}
          variant="primary"
          size="md"
          style={{ flexShrink: 0 }}
        >
          {isUpdating ? "Updating..." : "Update"}
        </Button>
        {hasManualOverride && (
          <Button
            onClick={handleResetToDefault}
            disabled={isUpdating}
            variant="tertiary"
            size="md"
            style={{ flexShrink: 0 }}
          >
            Reset to Default
          </Button>
        )}
      </Box>

      {message && (
        <Text
          as="p"
          size="xs"
          style={{
            color: message.type === "success" ? "#00d4aa" : "#ff3864",
            margin: 0,
          }}
        >
          {message.text}
        </Text>
      )}
    </Box>
  );
};

