import React from 'react';
import {Howl, Howler} from 'howler';

class HelloWorld extends React.Component {
  constructor(props) {
    super(props);
    this.state = this.getInitialState();
  }

  getInitialState() {
    return {
      howl: false,
      title: false,
      artist: false,
      album: false,
      file: false,
    };
  }

  saySomething(something) {
    console.log(something);
  }

  handleRandomClick(e) {
    this.getAndPlay();
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

  stop() {
    if (this.state.howl instanceof Howl && this.state.howl.playing()) {
      this.state.howl.stop();
    }
  }

  play(url) {
    this.stop();

    this.saySomething(url);

    let self = this;

    this.state.howl = new Howl({
      src: [url],
      onplayerror: function() {
        sound.once('unlock', function() {
          sound.play();
        });
      },
      onend: function() {
        self.stop();
        self.getAndPlay();
      }
    });

    this.state.howl.play();
  }

  getAndPlay() {
    let self = this;
    var request = new XMLHttpRequest();
    request.open('GET', this.getUrl('random'), true);
    request.onload = function() {
      if (this.status == 200) {
        let resp = this.response;
        let obj = JSON.parse(resp); 
        const title = obj.data[0].title;
        const artist = obj.data[0].artist;
        const album = obj.data[0].album;
        const file = obj.data[0].file_name;
        self.setState({
          artist: artist.length > 0 ? artist : false,
          title: title.length > 0 ? title : false,
          album: album.length > 0 ? album : false,
          file: file.length > 0 ? file : false,
        });

        let url = self.getUrl('play' + obj.data[0].path);
        self.play(url);
      }
    }

    request.send();
  }

  render() {
    console.log(this.state.file);

    let file;
    if (this.state.file) {
      file = <p>File: <span id="file">{this.state.file}</span></p>
    }

    let title;
    if (this.state.title) {
      title = <p>Title: <span id="title">{this.state.title}</span></p>
    }

    let artist;
    if (this.state.artist) {
      artist = <p>Artist: <span id="artist">{this.state.artist}</span></p>
    }

    let album;
    if (this.state.album) {
      album = <p>Album: <span id="album">{this.state.album}</span></p>
    }

    return (
      <div className="container">
        <h1>randomsound.uk</h1>
        <button onClick={this.handleRandomClick.bind(this)} id="roll">Roll the dice...</button>
        {file}
        {title}
        {artist}
        {album}
      </div>
    );
  }
}

export default HelloWorld;