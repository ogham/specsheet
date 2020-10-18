#!/usr/bin/env ruby
#
# This script runs Specsheet’s extended tests.
# You can pass it `--offline` to skip the ones that require a network.


require 'pathname'

unless Pathname('/vagrant').directory?
  $stderr.puts "This script is intended to be run on the Vagrant machine."
  exit 2
end

Dir.chdir('/vagrant/xtests')
$failures = []
$start_time = Time.now

$tick  = "\e[32m✔︎\e[0m"
$cross = "\e[1;31m✘\e[0m"
$binary = "#{ENV['HOME']}/target/debug/specsheet"


# Running functions

def specsheet_commands(*args)
  `#{$binary} --list-commands --colour always #{args.join(' ')}`
end

def specsheet_execute(*args)
  `#{$binary} -sexpand --colour always #{args.join(' ')}`
end

def run_simple_test
  yield
  if $?.success? || $?.exitstatus == 3
    print $tick
  else
    print $cross
  end
end

def run_test(output, file)
  file = Pathname(file)
  if output == file.read
    print $tick
  else
    print $cross
    $failures << file
    file.write(output)
  end
end


# Initial tests

print "\e[1mInitial tests:\e[0m "
run_simple_test { specsheet_execute }
run_simple_test { specsheet_execute("--help") }
run_simple_test { specsheet_execute("--version") }
run_test(specsheet_execute("load-tests/name-and-tags.toml"), 'load-tests/name-and-tags-output.txt')
run_test(specsheet_execute("--list-tags", "load-tests/name-and-tags.toml"), 'load-tests/list-tags-output.txt')
run_test(specsheet_execute("non-existent-file.txt"), 'load-tests/non-existent-file-output.txt')
run_test(specsheet_execute("unreadable"), 'load-tests/unreadable-file-output.txt')
run_test(specsheet_execute("-", "<", 'check-tests/fs/files.toml'), 'load-tests/stdin-output.txt')
puts


# Check tests

print "\e[1mCheck tests:\e[0m "
threads = Pathname('check-tests').glob('*/*.toml').map do |test_file|
  Thread.new do
    run_test(specsheet_commands(test_file), test_file.sub_ext('-commands.txt'))
    run_test(specsheet_execute(test_file), test_file.sub_ext('-output.txt'))
  end
end
threads.compact.each { |t| t.join }
puts


# Tests with rewrites

print "\e[1mRewrite tests:\e[0m "
  test_file = Pathname('rewrite-tests/http/https-downgrade.toml')
  opts = %w[ -R "https://example.org/->http://localhost:2002/" ]
  run_test(specsheet_commands(test_file, *opts), test_file.sub_ext('-commands.txt'))
  run_test(specsheet_execute( test_file, *opts), test_file.sub_ext('-output.txt'))
puts


# Network tests

if ARGV.include?('--offline')
  print "\e[1mNetwork tests:\e[0m (skipped)"
else
  print "\e[1mNetwork tests:\e[0m "
  threads = Pathname('network-tests').glob('*/*.toml').map do |test_file|
    Thread.new do
      run_test(specsheet_commands(test_file), test_file.sub_ext('-commands.txt'))
      run_test(specsheet_execute(test_file), test_file.sub_ext('-output.txt'))
    end
  end
  threads.compact.each { |t| t.join }
end
puts


# Result counts

puts
time_taken = (Time.now - $start_time).to_i
if $failures.empty?
  puts "All OK (in #{time_taken}s)"
else
  if $failures.length == 1
    puts "1 failed test (in #{time_taken}s):"
  else
    puts "#{$failures.length} failed tests (in #{time_taken}s):"
  end

  $failures.each do |path|
    puts "- #{path}"
  end

  exit 1
end
