Future Features

merge strategy push main... probably not wise.

"depends on " makes ticket depend on another ticket spec. for example Ready or Implmented.
epic, group tickets and enforce that only epic is done before other ticket  start

introduce a review process by an "agent supervisor". a sort of lead architect role that review PRs or merges and reject/ammend/accept automatically.

user management: we need to be able to assign tickets/epic to users. users are from github, but this could be configured differently, so must be part of config. server login should also be an oauth flow with github, again configurable. then add filter on view to see users ticket, another user's ticket, all tickets.


1. username is in frontmatter. it's different from agent name.
2. agent name hasn't proven very useful: many one time workers instead of long running multi task agents. remove it
2. creator is automatically assigned
3. username should be realuser (if not git host, config could contain user list), except one "unassigned" placeholder.
4. plugins and configuratin allow apm to use a git host (starting with github) usernames and authentication
5. apm-serve supports https and a simple to setup auth scheme, hopefully better than simply username/password.
