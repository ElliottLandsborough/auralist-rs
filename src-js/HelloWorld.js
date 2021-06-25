import React from 'react';
import {Howl, Howler} from 'howler';

class HelloWorld extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      date: new Date()
    };
  }

  saySomething(something) {
    console.log(something);
  }

  handleRandomClick(e) {
    this.saySomething("element clicked");
  }

  componentDidMount() {
    this.saySomething("component did mount");
  }

  render() {
    return (
      <div>
        <h1>Hello, world!</h1>
        <h2>It is {this.state.date.toLocaleTimeString()}.</h2>
        <button onClick={this.handleRandomClick.bind(this)}>Random Track</button>
      </div>
    );
  }
}

export default HelloWorld;