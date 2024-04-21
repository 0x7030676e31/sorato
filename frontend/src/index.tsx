import { render } from "solid-js/web";
import { Router, Route } from "@solidjs/router";

import App from "./app";
import "./index.scss";

import Clients from "./components/clients";
import Library from "./components/library";

render(
  () => (
    <Router root={App}>
      <Route path="sorato" component={Clients} />
      <Route path="sorato/library" component={Library} />
    </Router>
  ),
  document.getElementById("root")!
);

// import './index.css'
// import App from './App'

// const root = document.getElementById('root')

// render(() => <App />, root!)
