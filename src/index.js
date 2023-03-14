let ws = new WebSocket('ws://10.0.0.22:8330');
ws.onopen = () => ws.send('hello');


// add event listner for document to load elements

document.addEventListener('DOMContentLoaded', () => {

      let button = document.getElementById('sender');
      let canvas = document.getElementById('canvas');
      let ctx = canvas.getContext('2d');


      let rand_color = `rgb(
            ${Math.floor(255 - 255*Math.random())},
            ${Math.floor(255 - 255*Math.random())},
            ${Math.floor(255 - 255*Math.random())}
      )`;


      // add event listner for mouse click over canvas
      let dofill = false;

      canvas.addEventListener("mousedown", (event) => {
            dofill = true;
      });
      canvas.addEventListener("mouseup", (event) => {
            dofill = false;
      });

      let mousepos = { x: 0, y: 0 };

      canvas.addEventListener("mousemove", (event) => {
            if (dofill) {
                  mousepos.x = event.offsetX;
                  mousepos.y = event.offsetY;
                  
            }
      });

      ws.onmessage = event => {
            let data = JSON.parse(event.data);
            ctx.fillStyle = data.color;
            ctx.fillRect(data.x, data.y, 5, 5);
      }

      // add event listner for button to send message to server
      button.addEventListener('click', () => {
            ws.send('hello from button');
      });

      setInterval(() => {
            if (dofill)
            {
                  ctx.fillStyle = rand_color;
                  ctx.fillRect(mousepos.x, mousepos.y, 5, 5);
                  ws.send(JSON.stringify({
                        x: mousepos.x,
                        y: mousepos.y,
                        color: rand_color
                  }));
            }
                  

      }, 1000 / 100);



})