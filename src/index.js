let ws = new WebSocket('ws://localhost:8330');
      ws.onmessage = event => alert('Message from server: ' + event.data);
      ws.onopen = () => ws.send('hello');