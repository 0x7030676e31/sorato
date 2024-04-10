import { render } from "solid-js/web";
import { Router, Route } from "@solidjs/router";

import App from "./app";
import "./index.scss";

render(
  () => (
    <Router root={App}>

    </Router>
  ),
  document.getElementById("root")!
);

// import './index.css'
// import App from './App'

// const root = document.getElementById('root')

// render(() => <App />, root!)
