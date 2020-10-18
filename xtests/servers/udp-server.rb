#!/usr/bin/env ruby
require 'socket'

Socket.udp_server_loop(2001) do |msg, msg_src|
  puts "Received message: #{ msg.inspect }"
  msg_src.reply("thanks")
end
