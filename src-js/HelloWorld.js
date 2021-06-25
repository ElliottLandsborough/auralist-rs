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
    this.playRandomTune();
  }

  componentDidMount() {
    this.saySomething("component did mount");
  }

  getUrl(path) {
    let domainPrefix = 'http://localhost:1337/';

    if (window.location.hostname === 'randomsound.uk') {
        domainPrefix = 'https://randomsound.uk/';
    }

    return domainPrefix + path;
  }

  playRandomTune() {
    let self = this;
    var request = new XMLHttpRequest();
    request.open('GET', this.getUrl('random'), true);
    request.onload = function() {
      if (this.status == 200) {
        let resp = this.response;
        let obj = JSON.parse(resp); 
        let title = obj.data[0].title;
        let artist = obj.data[0].artist;
        let album = obj.data[0].album;
        let file_name = obj.data[0].file_name;
        let path = self.getUrl('play' + obj.data[0].path);
        self.saySomething(path);
      }
    }

    request.send();
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