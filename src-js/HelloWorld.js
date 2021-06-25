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

  getUrl(path) {
    let domainPrefix = 'http://localhost:1337/';

    if (window.location.hostname === 'randomsound.uk') {
        domainPrefix = 'https://randomsound.uk/';
    }

    return domainPrefix + path;
  }

  playRandomTune() {
    var request = new XMLHttpRequest();
    request.open('GET', getUrl('random'), true);
    request.onload = function() {
      if (this.status == 200) {
        let resp = this.response;
        let obj = JSON.parse(resp); 
        let title = obj.data[0].title;
        let artist = obj.data[0].artist;
        let album = obj.data[0].album;
        let file_name = obj.data[0].file_name;
        let path = obj.data[0].path.replace('/home/ubuntu/music-sorted/', 'https://randomsound.uk/files/');
        let audio = document.getElementById('audio');
        let source = document.getElementById('audioSource');
        source.src = path;
        audio.load();
        audio.play();
        document.querySelector('#title').innerHTML = title;
        document.querySelector('#artist').innerHTML = artist;
        document.querySelector('#album').innerHTML = album;
        document.querySelector('#file').innerHTML = file_name;
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