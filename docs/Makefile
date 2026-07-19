
PANDOC ?= pandoc
NAME ?= timestitch
INPUTFMT = markdown+pandoc_title_block

%.1: %.1.md
	$(PANDOC) --standalone $(PANDOCFLAGS) --from $(INPUTFMT) --to man -V section=1 $< -o $@

%.5: %.5.md
	$(PANDOC) --standalone $(PANDOCFLAGS) --from $(INPUTFMT) --to man -V section=5 $< -o $@


all: $(NAME).1 $(NAME).5

clean:
	rm $(NAME).1
	rm $(NAME).5

.PHONY: all

