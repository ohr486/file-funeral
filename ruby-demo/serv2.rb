require "socket"

server = TCPServer.new("localhost", 2345)

loop do
  socket = server.accept
  STDERR.puts "リクエストがきました"

  http_request = ""
  while (line = socket.gets) && (line != "\r\r")
    http_request += line
  end
  STDERR.puts http_request
  socket.close
end
