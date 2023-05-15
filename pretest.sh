#!/bin/bash

chown root:wheel test_assets/root_file.txt
chown root:wheel test_assets/root_file_excluded.txt
tmutil addexclusion test_assets/root_file_excluded.txt