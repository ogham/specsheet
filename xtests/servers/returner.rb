#!/usr/bin/env ruby

code = ARGV.shift or raise "Pass in a code to return"
exit code.to_i
