import express, {Request, Response} from "express"
import { WebSocketServer } from 'ws';

import * as Ockam from ".."

export * from "../worker";
export * from "../routing";
export * from "../node";

const app = express()
const server = require('http').createServer(app)

const wss = new WebSocketServer({ server: server});
const WebSocket = require('ws');

export class Hop implements Ockam.Worker {
  handleMessage(context: Ockam.Context, message: Ockam.Message) {
    console.log(context.address, " - received - ", message)
    // remove my address from beginning of onwardRoute
    message.onwardRoute.shift();
    // add my own address to beginning of returnRoute
    message.returnRoute.unshift(context.address);
    context.route(message)
  }
}

export class Printer implements Ockam.Worker {
  handleMessage(context: Ockam.Context, message: Ockam.Message) {
    console.log(context.address, " - received - ", message)
  }
}

export class Echoer implements Ockam.Worker {
  handleMessage(context: Ockam.Context, message: Ockam.Message) {
    console.log(context.address, " - received - ", message)
    // make returnRoute of incoming message, onwardRoute of outgoing message
    message.onwardRoute = message.returnRoute;
    // make my address the the returnRoute of the new message
    message.returnRoute = [context.address]
    context.route(message)
  }
}

// this gets triggered whenever a new connection is made to the server
wss.on('connection', function connection(ws) {
    console.log(`A new client is connected!`)
    ws.send('Welcome New Client!');

    let node = new Ockam.Node()

    node.startWorker("echoer", new Echoer())
    node.startWorker("h1", new Hop())
    node.startWorker("h2", new Hop())
    node.startWorker("h3", new Hop())

    node.startWorker("app", (context: Ockam.Context, message: Ockam.Message) => {
      ws.on('message', function incoming(message) {
        console.log(context.address, " - received - ", message)
  
        // take the websocket message and go through every client
        // then foreach client it would check if connection is open
        // if connection is open then it would send the message so this could work with multiple clients easily
        wss.clients.forEach(function each(client) {
          if (client !== ws && client.readyState === WebSocket.OPEN) {
            client.send(message);
          }
        });
      });
    })
    node.route({ onwardRoute: ["h1", "h2", "h3", "echoer"], returnRoute: ["app"], payload: "hello" })
});


app.get('/', (req: Request, res: Response) => res.send('Hello Ockam'))

server.listen(4000, () => console.log(`Listening on port: 4000`))
