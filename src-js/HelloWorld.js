import 'style.css';
import React from 'react';
import {Howl, Howler} from 'howler';
import MilkDrop from './MilkDrop';
import loadingAnimationSvg from '../images/loading.svg';
import loadingHypnoSvg from '../images/hypnotize.svg';
import Marquee from "react-fast-marquee";

async function delay(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  })
}

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
      ext: false,
      artist: '',
      title: '',
      album: '',
      file: '',
      playing: false,
      analyser: false,
      context: false,
      audio: false,
      soundID: false,
      thinking: false,
      includeSongs: true,
      includeMixes: true,
      hypnotize: true,
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

  enableVisualsHandler() {
    this.setState({enableVisuals: true});
  }

  enableMixes() {
    this.setState({includeMixes: true});
    this.setState({includeSongs: false});
  }

  enableSongs() {
    this.setState({includeMixes: false});
    this.setState({includeSongs: true});
  }

  enableBoth() {
    this.setState({includeMixes: true});
    this.setState({includeSongs: true});
  }

  searchForFile() {
    // todo: duckduckgo link
    // window.open(url,'_blank');
    // https://duckduckgo.com/?q=search
  }

  stop() {
    if (this.isPlaying() || this.state.howl) {
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
      onplayerror: function(sound) {
        console.log('play error');
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

  getAndPlay(alreadyRetried = 0) {
    let self = this;
    self.setState({thinking: true});
    var request = new XMLHttpRequest();
    request.timeout = 2000; // time in milliseconds
    let url = 'random/all';
    if (!this.state.includeSongs && this.state.includeMixes) {
      url = 'random/mixes';
    } else if (this.state.includeSongs && !this.state.includeMixes) {
      url = 'random/songs-and-tunes';
    }
    request.open('GET', this.getUrl(url), true);
    // todo: find out where the lock is. This doesn't work as a fix really.
    request.ontimeout = (e) => {
      console.log('Timeout :(');
      console.log('Retry number: ' + alreadyRetried);
      if (alreadyRetried !== 2) {
        self.stop();
        self.getAndPlay(alreadyRetried + 1)
      } else {
        self.stop();
      }
    };
    request.onload = function() {
      if (this.status == 200) {
        let resp = this.response;
        let obj = JSON.parse(resp); 
        const path = obj.data[0].path;
        const ext = obj.data[0].ext;
        const artist = "artist" in obj.data[0] ? obj.data[0].artist : '';
        const title = "title" in obj.data[0] ? obj.data[0].title : '';
        const album = "album" in obj.data[0] ? obj.data[0].album : '';
        const file = "file" in obj.data[0] ? obj.data[0].file : '';
        self.setState({
          path: path,
          ext: ext,
          artist: artist,
          title: title,
          album: album,
          file: file,
        });
        let url = self.getUrl('stream/' + obj.data[0].path);
        self.stop();
        self.play(url, ext);
      }
      // Wait before allowing another click
      setTimeout(
        () => self.setState({ thinking: false }), 
        2000
      );
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

    let loadingAnimation = <img src={loadingAnimationSvg}></img>
    let hypnotize = <img src={loadingHypnoSvg}></img>
    let hypnotizer = (this.state.playing ? <div class="hypnotizer" onClick={this.enableVisualsHandler.bind(this)}>{hypnotize}</div> : '')

    let playNextSong = (<a onClick={this.state.thinking ? null : this.handleRandomClick.bind(this)} className="play">{this.state.thinking ? loadingAnimation : "RANDOM"}</a>)

    let stop = (<a onClick={this.state.thinking ? null : this.handleStopClick.bind(this)} className="stop">STOP</a>)

    let mixes = (<div class={!this.state.includeSongs && this.state.includeMixes ? 'mixes enabled' : 'mixes disabled'} onClick={this.enableMixes.bind(this)}>Mixes</div>)
    let songs = (<div class={this.state.includeSongs && !this.state.includeMixes ? 'songs enabled' : 'songs disabled'} onClick={this.enableSongs.bind(this)}>Songs</div>)
    let both = (<div class={this.state.includeSongs && this.state.includeMixes ? 'both enabled' : 'both disabled'} onClick={this.enableBoth.bind(this)}>Both</div>)

    let marquee = (
      <Marquee>
        <span>Artist: {this.state.artist ? ' ' + this.state.artist : ' n/a'}</span>
        <span>Title: {this.state.title ? ' ' + this.state.title : ' n/a'}</span>
        <span>Album: {this.state.album ? ' ' + this.state.album : ' n/a'}</span>
        <span>File: {this.state.file ? ' ' + this.state.file : ' n/a'}</span>
      </Marquee>
    )

    return (
      <div className="container">
        <div className="controls">
          <div class="mixes-or-songs">{mixes}{songs}{both}</div>
          <div class="marquee" onClick={this.searchForFile.bind(this)}>
            {marquee}
          </div>
          <div class="play-container">
            {hypnotizer}
            {playNextSong}
          </div>
          {stop}
        </div>
        <div className="search">
        </div>
        <div class="milkdrop">
          {milkDrop}
        </div>
      </div>
    );
  }
}

export default HelloWorld;