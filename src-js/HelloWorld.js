import 'style.css';
import React from 'react';
import {Howl, Howler} from 'howler';
import MilkDrop from './MilkDrop';
import butterchurnPresets from 'butterchurn-presets';

class HelloWorld extends React.Component {
  constructor(props) {
    super(props);
    this.state = this.getInitialState();
  }

  componentDidMount() {
    document.title = "randomsound.uk";
  }

  getInitialState() {
    return {
      howl: false,
      title: false,
      artist: false,
      album: false,
      file: false,
      playing: false,
      context: false,
      audio: false,
      soundID: false,
    };
  }

  handleRandomClick(e) {
    this.getAndPlay();
  }

  handleStopClick(e) {
    this.stop();
    this.reportPlayState();
  }

  getUrl(path) {
    let domainPrefix = 'http://localhost:1337/';

    if (window.location.hostname === 'randomsound.uk') {
        domainPrefix = 'https://randomsound.uk/';
    }

    return domainPrefix + path;
  }

  isPlaying() {
    return this.state.howl instanceof Howl && this.state.howl.playing();
  }

  reportPlayState() {
    const isPlaying = this.isPlaying();

    this.setState(
      {
        playing: isPlaying,
        context: isPlaying ? Howler.ctx : false,
        audio: isPlaying ? this.state.howl._soundById(this.state.soundID) : false
      }
    );
  }

  stop() {
    if (this.isPlaying()) {
      this.state.howl.stop();
    }
  }

  play(url) {
    this.stop();

    let self = this;

    this.state.howl = new Howl({
      src: [url],
      onplayerror: function() {
        sound.once('unlock', function() {
          sound.play();
        });
      },
      onplay: function() {
        self.reportPlayState();
      },
      onend: function() {
        self.stop();
        self.getAndPlay();
      }
    });

    let soundID = this.state.howl.play();

    this.setState({soundID: soundID});
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

    let stop;
    if (this.state.playing) {
      stop = <a onClick={this.handleStopClick.bind(this)} className=" button stop">Stop</a>
    }

    let milkDrop;
    if (this.state.playing) {
      milkDrop = <MilkDrop width="400" height="300" context={this.state.context} audio={this.state.audio} />
    }

    return (
      <div className="container">
        <h1>randomsound.uk</h1>
        <div className="controls">
          <a onClick={this.handleRandomClick.bind(this)} className=" button play">Play / next</a>
          {stop}
        </div>
        {file}
        {title}
        {artist}
        {album}
        <div className="search">

        </div>
        <div className="milk-drop">
          {milkDrop}
        </div>
      </div>
    );
  }
}

export default HelloWorld;