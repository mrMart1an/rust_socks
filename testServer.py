import socket

host = '127.0.0.1'
port = 50000
backlog = 5 
size = 1024 
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM) 
s.bind((host,port)) 
s.listen(backlog) 
try:
    while 1: 
        client, address = s.accept() 
        data = client.recv(size)
        data = data.upper()
        if data: 
            client.send(data) 
        client.close()

except KeyboardInterrupt:
    s.close()