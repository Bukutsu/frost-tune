PREFIX ?= /usr
LIBEXECDIR ?= $(PREFIX)/libexec/frost-tune
POLICYDIR ?= $(PREFIX)/share/polkit-1/actions

.PHONY: install
install:
	mkdir -p $(DESTDIR)$(LIBEXECDIR)
	mkdir -p $(DESTDIR)$(POLICYDIR)
	install -m 755 target/release/frost-tune-hid-helper $(DESTDIR)$(LIBEXECDIR)/
	install -m 644 packaging/linux/org.frosttune.hid.policy $(DESTDIR)$(POLICYDIR)/
