import express, {Request, Response} from "express"
import { WebSocketServer } from 'ws';


const app = express()
const server = require('http').createServer(app)

const wss = new WebSocketServer({ server: server});
const WebSocket = require('ws');

// this gets triggered whenever a new connection is made to the server
wss.on('connection', function connection(ws) {
    console.log(`A new client is connected!`)
    ws.send('Welcome New Client!');

    ws.on('message', function incoming(message) {
      console.log('received: %s', message);

      // take the websocket message and go through every client
      // then foreach client it would check if connection is open
      // if connection is open then it would send the message so this could work with multiple clients easily
      wss.clients.forEach(function each(client) {
        if (client !== ws && client.readyState === WebSocket.OPEN) {
          client.send(message);
        }
      });
    });
});


app.get('/', (req: Request, res: Response) => res.send('Hello Ockam'))

server.listen(4000, () => console.log(`Listening on port: 4000`))
