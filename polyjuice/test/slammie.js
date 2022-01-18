const Slammie = artifacts.require("Slammie");

/*
 * uncomment accounts to access the test accounts made available by the
 * Ethereum client
 * See docs: https://www.trufflesuite.com/docs/truffle/testing/writing-tests-in-javascript
 */
contract("Slammie", function (/* accounts */) {
  it("should assert true", async function () {
    await Slammie.deployed();
    return assert.isTrue(true);
  });
});
