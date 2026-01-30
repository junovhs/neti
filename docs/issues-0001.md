# issues-0001: Initial Issue Consolidation

---

## FORMAT (DO NOT MODIFY)

**Status values:** `OPEN`, `IN PROGRESS`, `DONE`, `DESCOPED`

**Issue format:**
```
## [N] Title
**Status:** OPEN
**Files:** list of files to modify

Description of the task.

**Resolution:** (fill when DONE) What was done, any notes.
```

**Instructions:**
- Work issues in order you feel is most important.
- Update status as you go
- Add **Resolution:** when completing
- Don't modify this FORMAT section
- Content below the line is the work. when done, archive in docs/archive and create next issues doc, either populated with issues are at least the template (issues-0002.md, issues-0003.md etc)

---

these are stubs, I need to flesh this out properly but dont want to forget 

1. I want slopchop to be smarter about how it presents clippy errors - so many times it outputs like 10k lines of the same **kind** of issue, that ultimately the AI i show it to is like "oh thats like two easy fixes" - the lesson: clippy error lengths does not equate to number of issues - so I want slopchop to be smarter about keep that maximally informative, BUT, still succinct. 
2. slopchop has a recurring issue with the clipboard, for example I see it in a few places but notably Cargo.toml files, the last character is ofte a # and it shows a carriage return error there, so the fix has been go in, delete that last character, manually retype it, clippy happy. broadly it seems there are some character conversion issues happening across the clipboard transfer process, we need to look at. 
3. with the advent of SEMMAP, i am thinking its best to cut all packing functinoality/context stuff. So yea, we need to cut that out. 
4. we need to consider the implications to the prompt mechanisms because we're cutting the pack stuff. we have AGENT-README.md in root, maybe thats enough
5. we need to reframe the "v2 analysis engine" as just the analysis engine, and ensure we have cut all traces of v1 stuff. makes no sense to keep calling it that. creates confusion. 
6. revisit law of locality, feels weirdly separate from the other stuff - should it be elevated so much?
7. I use slopchop to apply code MY way, which, is most of the time, chat based. like, a web browser chat page. it is not without its challenges. but I like the purposeful, forced human in the loop turn based thing. thats really it - its like a turn based RPG - its just VERY CLEAR whos turn it is, and it forces you to respond. whatever though. AAAAAAALL that said... I really think its important this be tuned for agentic platforms like Google Antigravity. That is where many of my audience will live i feel. but lets think about how im using this app and how i think others will. so me, I ask AI to write code in a chat window on the left, and I have my terminal and code editor (zed) on the right - on the left AI gives me code, i hit "copy", move over to the right, in the terminal CD'd into the root of my project i type 'slopchop apply' and it applies the code files in the right place all parsed up correctly and rejecting backticks and stuff, the artifacts you often see with chats. most of the time anyway.it runs through the whole checks system - so I am quality checking code on an atomic level with every file save basically, whereas I imagine other devs will hook this up into their CI workflow on github actions or something WHEN THEY PUSH. so im doing it like a level lower. i just want to make sure slopchop is focused on what it actually is, not a monolith. i know the apply system and the checks are deeply tied, but should they be separate? I genuinely do not know. 
8. oh yea we can definitely cut slopchop map and map --deps too, becuse I have that in another app and its way better.
