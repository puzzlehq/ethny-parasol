// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import "sunscreen/src/FHE.sol";

contract Ballot {
    struct Proposal {
        string name;
        string contents;
        bytes voteCount;
    }

    struct Voter {
        bool hasVoted;
        bytes[] votes;
    }

    mapping(address => Voter) private voters;
    FHE fhe;
    Proposal[] public proposals;

    constructor() {
        fhe = new FHE();
    }

    function getPublicKey() public view returns (bytes memory) {
        return fhe.networkPublicKey();
    }

    function getProposals() public view returns (Proposal[] memory) {
        return proposals;
    }

    function addProposal(string memory name, string memory contents) public {
        bytes memory zero = fhe.encryptUint64(0);
        proposals.push(Proposal(name, contents, zero));
    }

    function vote(bytes[] memory votes) public {
        Voter storage sender = voters[msg.sender];
        require(!sender.hasVoted, "Already voted.");
        require(
            votes.length == proposals.length,
            "You need to give exactly as many votes as proposals"
        );
        sender.hasVoted = true;
        sender.votes = votes;

        for (uint i = 0; i < proposals.length; i++) {
            proposals[i].voteCount = fhe.addUint64EncEnc(
                fhe.networkPublicKey(),
                proposals[i].voteCount,
                sender.votes[i]
            );
        }
    }
    function getProposalTallys(
        bytes calldata reencPublicKey
    ) public view returns (bytes[] memory) {
        bytes[] memory reEncProposals = new bytes[](proposals.length);
        for (uint i = 0; i < proposals.length; i++) {
            reEncProposals[i] = fhe.reencryptUint256(
            reencPublicKey,
            proposals[i].voteCount);
        }

        return reEncProposals;
    }
}
