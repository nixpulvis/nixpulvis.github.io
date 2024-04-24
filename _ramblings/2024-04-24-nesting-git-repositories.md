---
layout: rambling
---

So I had an idea last night... what if we could simply nest git repositories
inside each other, without all the added complexity of `git-submodule`?

Consider for a moment a scenario like the one I have. During my years at school
I collected a bunch of repos for each class e.g. `cs2500`, each with their own
history. Then, I wanted to put them all together under a new repo named
`school`. A perfect use case for submodules, no? But the issue is I didn't want
to manage updating two repos and remotes every time I touched anything.
Wouldn't it be nice if all I needed to do was `mv cs2500 school` and git could
handle the nested `.git` directory as a sort of subrepository.

With this setup, I imagine history would simply be the union of all
subrepositories' histories, and a commit to `cs2500` would only change the
contents of `cs2500/.git`. At any point I should be able to move back out the 
subrepository with `mv school/cs2500 cs2500` and have all my progress
persevered.

This is just the very beginnings of an idea, so I expect there are a lot of
edge cases and blatant issues with it, but I do think there could be something
here worth thinking about. People often shy away from using submodules because
of the overhead they add. So giving a new way to manage nested repositories
could go a long way for organizing things inside git.
