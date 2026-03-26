Let's create a new spec with these following guidelines and requirements:
  1. apm should have the same UI/UX feel as linear: fast, immediate, simple
  2. it is aimed for teams of engineers workign with agents. The team is responsible for the whole code base, but at any time there is likely only one
  engineer working on a repo or set of repo. apm makes it easy for another engineer to step in and continue the work.
  3. it's using git to be fully distributed. no other database. All tickets are recorded in the repo, and their version history is the record of
  activity. for multi repo setups, tickets can be saved in on of the repo, or in a dedicated repo.
  4. apm has a fast way to rebuild it's state by loading all non closed tickets. for investigation, it can load closed tickets in a slower process.
  5. it allow agents to be fully autonomous by separating their work in branches dedicated to the ticket.
  6. tickets are saved as a document containing a set of predetermined fields, a spec that evolves with the work, and a conversation with the user to
  resolve questions before getting to the final spec.
  7. a set of CLI tools allow fool usage of the system even without using a client.
  8. a web based client connects to the github repo for remote work

  
one possible correction: ticket is created in main, but, once picked up all the changes to it are commited to the dedicated branch. weigh the pros and cons
to avoid problems with manual changes, a tool is provided to verify the integrity of the changes. that tool can be setup as a pre-commit hook
one central document (apm.settings?) describes the state flow, the repos managed, and other customizable features.
part of settings is a set of agent settings (Claude.md etc) taht help. for instance each agent/subagent should have name-agent.
does it make sense for the local client (even cli) to have a sqlite db for temp data, that gets refreshed automatically or by triggering git pull. 
