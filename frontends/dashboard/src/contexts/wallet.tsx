import { Buffer } from 'buffer';
import { bech32 } from 'bech32';
import { createContext, useContext, useState, useEffect } from 'react';
import type { ReactNode } from 'react';

// Cardano wallet API types based on CIP-30
export interface CardanoAPI {
  enable(): Promise<CardanoWalletAPI>;
  isEnabled(): Promise<boolean>;
  apiVersion: string;
  name: string;
  icon: string;
}

export interface CardanoWalletAPI {
  getBalance(): Promise<string>;
  getUsedAddresses(): Promise<string[]>;
  getUnusedAddresses(): Promise<string[]>;
  getChangeAddress(): Promise<string>;
  getRewardAddresses(): Promise<string[]>;
  getUtxos(): Promise<string[]>;
  signTx(tx: string, partialSign: boolean): Promise<string>;
  signData(address: string, payload: string): Promise<{ signature: string; key: string; }>;
  submitTx(tx: string): Promise<string>;
}

export interface CardanoWallet {
  name: string;
  icon: string;
  apiVersion: string;
  key: string;
  changeAddress?: string;
}

interface WalletContextType {
  availableWallets: CardanoWallet[];
  connectedWallet: CardanoWalletAPI | null;
  connectedWalletDetails: CardanoWallet | null;
  isConnecting: boolean;
  error: string | null;
  connectWallet: (walletKey: string) => Promise<void>;
  disconnectWallet: () => void;
  refreshWallets: () => void;
}

const WalletContext = createContext<WalletContextType | undefined>(undefined);

enum NetworkId {
  MAINNET = 1,
  TESTNET = 0,
}

function addressToBech32(hex: string): string {
  const hexAddress = hex.toLowerCase();
  const addressType = hexAddress.charAt(0);
  const networkId = Number(hexAddress.charAt(1)) as NetworkId;
  const addressBytes = Buffer.from(hexAddress, 'hex');
  const words = bech32.toWords(addressBytes);
  let prefix = ['e', 'f'].includes(addressType) ? 'stake' : 'addr';
  if (networkId === NetworkId.TESTNET) {
    prefix += '_test';
  }

  return bech32.encode(prefix, words, 1000);
}

export function WalletProvider({ children }: { children: ReactNode; }) {
  const [availableWallets, setAvailableWallets] = useState<CardanoWallet[]>([]);
  const [connectedWallet, setConnectedWallet] = useState<CardanoWalletAPI | null>(null);
  const [connectedWalletDetails, setConnectedWalletDetails] = useState<CardanoWallet | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Detect available Cardano wallets
  const detectWallets = () => {
    if (typeof window === 'undefined' || !window.cardano) {
      return [];
    }

    const wallets: CardanoWallet[] = [];

    Object.entries(window.cardano || {}).forEach(([key, walletAPI]) => {
      if (walletAPI) {
        wallets.push({
          name: walletAPI.name || key,
          icon: walletAPI.icon || '',
          apiVersion: walletAPI.apiVersion || '1.0.0',
          key,
        });
      }
    });

    return wallets;
  };

  // Initialize and detect wallets
  const refreshWallets = () => {
    const wallets = detectWallets();
    setAvailableWallets(wallets);
  };

  useEffect(() => {
    // Initial detection
    refreshWallets();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Connect to a specific wallet
  const connectWallet = async (walletKey: string) => {
    if (typeof window === 'undefined' || !window.cardano) {
      setError('Cardano wallet API not available');
      return;
    }

    setIsConnecting(true);
    setError(null);

    try {
      const walletAPI = window.cardano?.[walletKey] as CardanoAPI | undefined;

      if (!walletAPI) {
        throw new Error(`Wallet ${walletKey} not found`);
      }

      // Request wallet connection
      const api = await walletAPI.enable();

      const changeAddress = addressToBech32(await api.getChangeAddress());

      setConnectedWallet(api);
      setConnectedWalletDetails({
        name: walletAPI.name,
        icon: walletAPI.icon,
        apiVersion: walletAPI.apiVersion,
        key: walletKey,
        changeAddress,
      });
      setError(null);

      // Store connection preference in localStorage
      localStorage.setItem('connectedWallet', walletKey);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to connect wallet';
      setError(errorMessage);
      // eslint-disable-next-line no-console
      console.error('Wallet connection error:', err);
    } finally {
      setIsConnecting(false);
    }
  };

  // Disconnect wallet
  const disconnectWallet = () => {
    setConnectedWallet(null);
    setConnectedWalletDetails(null);
    setError(null);
    localStorage.removeItem('connectedWallet');
  };

  // Auto-reconnect to previously connected wallet
  useEffect(() => {
    const previouslyConnected = localStorage.getItem('connectedWallet');

    if (previouslyConnected && availableWallets.length > 0) {
      const walletExists = availableWallets.some(w => w.key === previouslyConnected);

      if (walletExists) {
        connectWallet(previouslyConnected);
      }
    }
  }, [availableWallets]);

  const value: WalletContextType = {
    availableWallets,
    connectedWallet,
    connectedWalletDetails,
    isConnecting,
    error,
    connectWallet,
    disconnectWallet,
    refreshWallets,
  };

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

// Custom hook to use the wallet context
export function useWallet() {
  const context = useContext(WalletContext);

  if (context === undefined) {
    throw new Error('useWallet must be used within a WalletProvider');
  }

  return context;
}

// Type augmentation for window.cardano
declare global {
  interface Window {
    cardano?: {
      [key: string]: CardanoAPI;
    };
  }
}
