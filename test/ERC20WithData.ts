// SPDX-License-Identifier: Apache-2.0
import { expect } from "chai";
import { ethers } from "hardhat";
import { MaxUint256, parseEther, toUtf8Bytes } from "ethers"; // Import MaxUint256
import { ERC20WithData } from "../typechain-types/contracts/ERC20WithData";
import { SignerWithAddress } from "@nomicfoundation/hardhat-ethers/signers";

describe("ERC20WithData", function () {
  let token: ERC20WithData;
  let owner: SignerWithAddress;
  let addr1: SignerWithAddress;
  let addr2: SignerWithAddress;
  let addrs: SignerWithAddress[];

  beforeEach(async function () {
    [owner, addr1, addr2, ...addrs] = await ethers.getSigners();

    const ERC20WithDataFactory = await ethers.getContractFactory(
      "ERC20WithData"
    );
    token = (await ERC20WithDataFactory.deploy(
      "TestToken",
      "TT"
    )) as ERC20WithData;
  });

  it("Should assign the total supply of tokens to the owner", async function () {
    const ownerBalance = await token.balanceOf(owner.address);
    expect(await token.totalSupply()).to.equal(ownerBalance);
  });

  it("Should mint tokens with data", async function () {
    const mintAmount = parseEther("100");
    const data = toUtf8Bytes("Mint data");

    await token.mintWithData(addr1.address, mintAmount, data);
    expect(await token.balanceOf(addr1.address)).to.equal(mintAmount);
  });

  it("Should fail minting tokens with data if not owner", async function () {
    const mintAmount = parseEther("100");
    const data = toUtf8Bytes("Mint data");

    await expect(
      token.connect(addr1).mintWithData(addr1.address, mintAmount, data)
    ).to.be.revertedWith("Ownable: caller is not the owner");
  });

  it("Should transfer tokens with data", async function () {
    const mintAmount = parseEther("100");
    await token.mintWithData(owner.address, mintAmount, "0x");

    const transferAmount = parseEther("10");
    const data = toUtf8Bytes("Transfer data");

    await token.transferWithData(
      owner.address,
      addr1.address,
      transferAmount,
      data
    );
    expect(await token.balanceOf(addr1.address)).to.equal(transferAmount);
    expect(await token.balanceOf(owner.address)).to.equal(
      mintAmount - transferAmount
    ); // Use - operator
  });

  it("Should transfer tokens with data using transferFrom", async function () {
    const mintAmount = parseEther("100");
    await token.mintWithData(owner.address, mintAmount, "0x");

    const approveAmount = MaxUint256; // Use MaxUint256 directly
    await token.approve(addr1.address, approveAmount);

    const transferAmount = parseEther("10");
    const data = toUtf8Bytes("Transfer data");

    await token
      .connect(addr1)
      .transferWithData(owner.address, addr2.address, transferAmount, data);

    expect(await token.balanceOf(addr2.address)).to.equal(transferAmount);
    expect(await token.balanceOf(owner.address)).to.equal(
      mintAmount - transferAmount
    ); // Use - operator
  });

  it("Should burn tokens with data", async function () {
    const mintAmount = parseEther("100");
    await token.mintWithData(owner.address, mintAmount, "0x");

    const burnAmount = parseEther("30");
    const data = toUtf8Bytes("Burn data");

    await token.burnWithData(owner.address, burnAmount, data);
    expect(await token.balanceOf(owner.address)).to.equal(
      mintAmount - burnAmount
    ); // Use - operator
  });

  it("Should fail burning tokens with data if not owner", async function () {
    const mintAmount = parseEther("100");
    await token.mintWithData(owner.address, mintAmount, "0x");

    const burnAmount = parseEther("30");
    const data = toUtf8Bytes("Burn data");

    await expect(
      token.connect(addr1).burnWithData(owner.address, burnAmount, data)
    ).to.be.revertedWith("ERC20WithData: caller is not owner");
  });

  it("Should approve tokens with data", async function () {
    const approveAmount = parseEther("50");
    const data = toUtf8Bytes("Approve data");

    await token.approveWithData(addr1.address, approveAmount, data);
    expect(await token.allowance(owner.address, addr1.address)).to.equal(
      approveAmount
    );
  });

  it("Should support IERC20WithData interface", async function () {
    expect(await token.supportsInterface("0xe9ca311f")).to.be.true;
  });
});
