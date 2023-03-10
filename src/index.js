let ws = new WebSocket('ws://localhost:8330');
ws.onmessage = event => alert('Message from server: ' + event.data);
ws.onopen = () => ws.send('hello');


// add event listner for document to load elements

document.addEventListener('DOMContentLoaded', () => {

      let button = document.getElementById('sender');

      button.addEventListener('click', () => {
      ws.send('hello from button');
      });

})