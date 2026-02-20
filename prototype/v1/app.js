function nav(active){
  return `
  <div class="top"><strong>Ovia Prototype v1</strong><span class="muted">Laptop-first clickable mock</span></div>
  <div class="layout">
    <aside class="nav">
      <a class="${active==='dashboard'?'active':''}" href="./dashboard.html">Dashboard</a>
      <a class="${active==='team'?'active':''}" href="./team-identity.html">Team Identity</a>
      <a class="${active==='person'?'active':''}" href="./person-360.html">Person 360</a>
      <a class="${active==='ask'?'active':''}" href="./ask.html">Ask Ovia</a>
      <a class="${active==='reports'?'active':''}" href="./reports.html">Reports</a>
      <a class="${active==='settings'?'active':''}" href="./settings.html">Settings</a>
    </aside>
    <main class="main">`;
}
function end(){return `</main></div>`}
