var i = 0;

var player = -1;

var ws;


self.onmessage = function (event) {
	if ( player == -1 ) {
		player = event.data;
		ws  = new WebSocket("ws://127.0.0.1:8001/"+player)
		ws.onmessage = function (event) {
			// got this from webserver
  			postMessage(event.data);
		};
	} else {
		if ( ws.readyState == 1 ) {
			ws.send(event.data);
			//console.log("Sennt "+event.data);
		}
	}
}
