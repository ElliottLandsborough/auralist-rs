import 'style.css';
import React from 'react';
import {Howl, Howler} from 'howler';
import MilkDrop from './MilkDrop';

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
      enableVisuals: false,
      width: 0,
      height: 0,
      howl: false,
      file: false,
      ext: false,
      playing: false,
      analyser: false,
      context: false,
      audio: false,
      soundID: false,
      thinking: false,
    };
  }

  handleRandomClick(e) {
    this.getAndPlay();
  }

  handleStopClick(e) {
    this.stop();
  }

  getUrl(path) {
    let domainPrefix = '';

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

    let analyser = false;
    if (this.isPlaying()) {
      analyser = Howler.ctx.createAnalyser();
      Howler.masterGain.disconnect();
      Howler.masterGain.connect(analyser);
    }

    this.setState(
      {
        playing: isPlaying,
        context: isPlaying ? Howler.ctx : false,
        analyser: isPlaying ? analyser : false,
        audio: isPlaying ? this.state.howl._soundById(this.state.soundID) : false
      }
    );
  }

  enableVisualsHandler(event) {
    console.log(event);
    console.log('hello world');
  }

  stop() {
    if (this.isPlaying()) {
      this.state.howl.stop();
      this.reportPlayState();
    }
  }

  play(url, ext) {
    this.stop();

    let self = this;

    this.state.howl = new Howl({
      src: [url],
      format: [ext],
      html5: true,
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
    self.setState({thinking: true});
    var request = new XMLHttpRequest();
    request.open('GET', this.getUrl('random'), true);
    request.onload = function() {
      if (this.status == 200) {
        let resp = this.response;
        let obj = JSON.parse(resp); 
        const path = obj.data[0].path;
        const ext = obj.data[0].ext;
        self.setState({
          path: path,
          ext: ext,
        });
        let url = self.getUrl('stream/' + obj.data[0].path);
        self.play(url, ext);
      }
      self.setState({thinking: false});
    }
    request.send();
  }

  render() {
    let milkDrop;
    if (this.state.playing && this.state.enableVisuals) {
      milkDrop = (
        <MilkDrop
          width={this.state.width}
          height={this.state.height}
          context={this.state.context}
          analyser={this.state.analyser}
          audio={this.state.audio}
          playing={this.isPlaying()}
        />
      )
    }

    let playNextSong;
    if (!this.state.thinking) {
      playNextSong = (<a onClick={this.state.thinking ? null : this.handleRandomClick.bind(this)} className="play">START / NEXT</a>)
    }

    let stop = (<a onClick={this.state.thinking ? null : this.handleStopClick.bind(this)} className="stop">STOP</a>)

    return (
      <div className="container">
        <div class="control" onClick={this.enableVisualsHandler}>enable</div>
        <div className="controls">
          {playNextSong}
          {stop}
        </div>
        <div className="search">
        </div>
        {milkDrop}
      </div>
    );
  }
}

export default HelloWorld;