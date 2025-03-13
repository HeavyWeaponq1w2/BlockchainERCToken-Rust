import { buildModule } from "@nomicfoundation/hardhat-ignition/modules";

const ERC20WithDataModule = buildModule("ERC20WithDataModule", (m) => {
  const ERC20WithData = m.contract("ERC20WithData", ["TESTTOKEN", "TTK"]);

  return { ERC20WithData };
});

export default ERC20WithDataModule;
