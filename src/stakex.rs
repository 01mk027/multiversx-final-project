#![no_std]
use multiversx_sc::imports::*;
use multiversx_sc::derive_imports::*;




#[multiversx_sc::contract]

pub trait StakingContract {
    #[init]
    fn init(&self, initial_apy: u64) {
        self.apy().set(initial_apy);
    }

    // Storage for user stakes
    #[storage_mapper("stakes")]
    fn stakes(&self, address: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[storage_mapper("last_stake_block")]
    fn last_stake_block(&self, address: &ManagedAddress) -> SingleValueMapper<u64>;

    #[storage_mapper("apy")]
    fn apy(&self) -> SingleValueMapper<u64>;

    // Stake EGLD tokens
    #[payable("EGLD")]
    #[endpoint(stake)]
    fn stake(&self) {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().egld_value();

        require!(*payment > 0, "Must stake a positive amount");

        let current_stake = self.stakes(&caller).get();//unwrap_or(BigUint::zero());
        let total_stake = &current_stake + &*payment;

        self.stakes(&caller).set(&total_stake);
        self.last_stake_block(&caller).set(self.blockchain().get_block_nonce());
    }

    // Unstake EGLD tokens
    #[endpoint(unstake)]
    fn unstake(&self, amount: BigUint) {
        let caller = self.blockchain().get_caller();
        let current_stake = self.stakes(&caller).get();

        require!(current_stake >= amount, "Insufficient staked amount");

        let remaining_stake = &current_stake - &amount;

        self.stakes(&caller).set(&remaining_stake);
        self.send().direct_egld(&caller, &amount);
    }

    // Claim rewards
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        let caller = self.blockchain().get_caller();
        let rewards = self.calculate_rewards(&caller);

        require!(rewards > 0, "No rewards available");

        self.last_stake_block(&caller).set(self.blockchain().get_block_nonce());
        self.send().direct_egld(&caller, &rewards);
    }

    // Calculate rewards
    #[view(calculateRewards)]
    fn calculate_rewards(&self, address: &ManagedAddress) -> BigUint {
        let staked_amount = self.stakes(address).get();
        let last_block = self.last_stake_block(address).get();
        let current_block = self.blockchain().get_block_nonce();

        let blocks_staked = current_block - last_block;
        let apy = self.apy().get();

        // Formula: Rewards = Staked Amount * APY * Duration / Blocks Per Year
        let rewards = staked_amount * apy * BigUint::from(blocks_staked) / BigUint::from(31_536_000u64); // Approx. blocks per year
        rewards
    }

    // Get staking position for a user
    #[view(getStakingPosition)]
    fn get_staking_position(&self, address: ManagedAddress) -> (BigUint, u64) {
        let staked_amount = self.stakes(&address).get();
        let last_block = self.last_stake_block(&address).get();
        (staked_amount, last_block)
    }

    // View current APY
    #[view(getApy)]
    fn get_apy(&self) -> u64 {
        self.apy().get()
    }

    // Update APY (admin-only)
    #[only_owner]
    #[endpoint(setApy)]
    fn set_apy(&self, new_apy: u64) {
        self.apy().set(new_apy);
    }
}
