.PHONY: test_dir_edit test_dir_save test_dir_abort cli.md

test_dir_edit:
	unzip test_assets/test_dir.zip -d test_assets

test_dir_save:
	rm -f test_assets/test_dir.zip && cd test_assets && zip -r test_dir.zip test_dir && rm -rf test_dir

test_dir_abort:
	rm -rf test_assets/test_dir

cli.md:
	cargo run -- markdown-help > cli.md