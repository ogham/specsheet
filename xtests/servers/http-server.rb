#!/usr/bin/env ruby
require 'webrick'


class MyServlet < WEBrick::HTTPServlet::AbstractServlet
  def do_GET(request, response)
    case request.path

    when '/type'
      response.status = 200
      response.content_type = request.query['ct']
      response.body = ''

    when '/content'
      response.status = 200
      response.content_type = 'text/plain'
      response.body = request.query['c'] + "\n"

    when '/status'
      response.status = request.query['s'].to_i
      response.content_type = 'application/json'
      response.body = "{}\n"

    when '/redirect'
      response.status = 302
      response['Location'] = request.query['l']

    when '/header'
      response.status = 200
      response.content_type = 'text/plain'
      response['X-Waffles'] = request['X-Waffles']

    when '/500-with-location'
      response.status = 500
      response['Location'] = request.query['l']

    else
      response.status = 404
    end
  end
end


server = WEBrick::HTTPServer.new(:Port => 2002)
server.mount '/', MyServlet
trap(:INT) { server.shutdown }
server.start
