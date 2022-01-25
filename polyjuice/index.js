import { providers, ContractFactory, Signer } from "ethers";
import { PolyjuiceHttpProvider } from "@polyjuice-provider/web3";

const polyjuiceConfig: PolyjuiceConfig = {
  abiItems: ['your abi items array'],
  web3Url: 'godwoken web3 rpc url', 
};

export async function createEthersSignerWithMetamask(): Promise<
  Signer | undefined
> {
  if ((window as any).ethereum) {
    const provider = new providers.Web3Provider(
      new PolyjuiceHttpProvider(polyjuiceConfig.web3Url!, polyjuiceConfig)
    );
    let signer;

    try {
      await (window as any).ethereum.enable();
      signer = provider.getSigner((window as any).ethereum.selectedAddress);
    } catch (error) {
      // User denied account access...
      throw error;
    }

    return signer;
  }

  console.error(
    "Non-Ethereum browser detected. You should consider trying MetaMask!"
  );
  return undefined;
}

const signer = await createEthersSignerWithMetamask();
const contract = new ethers.Contract(
  'your contract address',
  'your contract abi',
  signer
);
let overrides = {
  gasLimit: 0x54d30,
  gasPrice: 0x0,
  value: 0x0,
};
const txResponse = await contract.METHOD_NAME(..args, overrides);
console.log(txResponse);