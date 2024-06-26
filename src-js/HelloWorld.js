import 'style.css';
import React from 'react';
import {Howl, Howler} from 'howler';
import MilkDrop from './MilkDrop';
import {LoadingSvg} from './LoadingSvg.jsx'
import {HypnotizeSvg} from './HypnotizeSvg.jsx';
import Marquee from "react-fast-marquee";

class HelloWorld extends React.Component {
  constructor(props) {
    super(props);
    this.state = this.getInitialState();
    this.updateWindowDimensions = this.updateWindowDimensions.bind(this);
  }
  
  componentDidMount() {
    document.title = "randomsound.uk";
    this.updateWindowDimensions();
    window.addEventListener('resize', this.updateWindowDimensions);

    let includeMixes = localStorage.getItem('includeMixes');
    let includeTunes = localStorage.getItem('includeTunes');

    this.setState({
      includeMixes: Object.is(includeMixes, null) ? true : JSON.parse(includeMixes),
      includeTunes: Object.is(includeTunes, null) ? false : JSON.parse(includeTunes),
    });
  }
  
  componentWillUnmount() {
    window.removeEventListener('resize', this.updateWindowDimensions);
  }
  
  updateWindowDimensions() {
    this.setState({ width: window.innerWidth, height: window.innerHeight });
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
      includeTunes: false,
      includeMixes: true,
      hypnotize: false,
    };
  }

  handleRandomClick(e) {
    this.getAndPlay();
  }

  handleStopClick(e) {
    this.stop();
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

  disableVisualsHandler() {
    this.setState({enableVisuals: false});
  }

  enableMixes() {
    this.setState({
      includeMixes: true,
      includeTunes: false,
    });
    localStorage.setItem('includeMixes', true);
    localStorage.setItem('includeTunes', false);
  }

  enableTunes() {
    this.setState({
      includeMixes: false,
      includeTunes: true,
    });
    localStorage.setItem('includeMixes', false);
    localStorage.setItem('includeTunes', true);
  }

  enableBoth() {
    this.setState({
      includeMixes: true,
      includeTunes: true,
    });
    localStorage.setItem('includeMixes', true);
    localStorage.setItem('includeTunes', true);
  }

  searchForFile() {
    // todo: duckduckgo link
    // window.open(url,'_blank');
    // https://duckduckgo.com/?q=search
  }

  stop() {
    if (this.isPlaying() || this.state.howl) {
      this.state.howl.unload();
    }
    this.reportPlayState();
    this.setState({
      thinking: false,
      hypnotize: false,
    });
  }

  play(url, ext) {
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
        self.setState({
          thinking: false,
          hypnotize: false,
        });
      },
      onplay: function() {
        self.reportPlayState();
        self.setState({
          thinking: false,
          hypnotize: true,
        });
      },
      onend: function() {
        console.log('File ended. Playing next random one...');
        self.getAndPlay();
      },
      onstop: function() {
        console.log('File stopped.');
      },
    });

    let soundID = this.state.howl.play();

    this.setState({soundID: soundID});
  }

  getAndPlay(alreadyRetried = 0) {
    let self = this;
    self.setState({
      thinking: true,
      hypnotize: false,
    });
    var request = new XMLHttpRequest();
    request.timeout = 10000; // time in milliseconds
    let url = 'random/all';
    if (!this.state.includeTunes && this.state.includeMixes) {
      url = 'random/mixes';
    } else if (this.state.includeTunes && !this.state.includeMixes) {
      url = 'random/tunes';
    }
    request.open('GET', url, true);
    // todo: find out where the lock is. This doesn't work as a fix really.
    request.ontimeout = (e) => {
      console.log('Timeout :(');
      console.log('Retry number: ' + alreadyRetried);
      if (alreadyRetried !== 2) {
        self.getAndPlay(alreadyRetried + 1)
      }
    };
    request.onload = function() {
      if (this.status == 200) {
        let resp = this.response;
        let obj = JSON.parse(resp);
        // todo: this is a bit messy, perhaps return a 404
        if (!resp || !obj || !("data" in obj)) {
          console.log("No data in response or response was invalid.");
          self.setState({
            thinking: false,
            hypnotize: false,
          });
          return;
        }
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
        let url = 'stream/' + obj.data[0].path;
        self.stop();
        self.play(url, ext);
      }
    }
    request.send();
  }

  render() {
    let milkDrop;

    if (this.state.playing && this.state.enableVisuals) {
      milkDrop = (
        <div class="milkdrop">
          <div class="close" onClick={this.disableVisualsHandler.bind(this)}></div>
          <MilkDrop
            width={this.state.width}
            height={this.state.height}
            context={this.state.context}
            analyser={this.state.analyser}
            audio={this.state.audio}
            playing={this.isPlaying()}
          />
        </div>
      )
    }

    let loadingAnimation = <LoadingSvg />
    let hypnotize = <HypnotizeSvg />

    let hypnotizer = (this.state.hypnotize && this.state.playing && !this.state.enableVisuals ? <div class="hypnotizer" onClick={this.enableVisualsHandler.bind(this)}>{hypnotize}</div> : '')

    let playNextSong = (<a onClick={this.state.thinking ? null : this.handleRandomClick.bind(this)} className="play">{this.state.thinking ? loadingAnimation : "RANDOM"}</a>)

    let stop = (<a onClick={this.state.thinking ? null : this.handleStopClick.bind(this)} className="stop">STOP</a>)

    let mixes = (<div class={!this.state.includeTunes && this.state.includeMixes ? 'mixes enabled' : 'mixes disabled'} onClick={this.enableMixes.bind(this)}>Mixes</div>)
    let tunes = (<div class={this.state.includeTunes && !this.state.includeMixes ? 'tunes enabled' : 'tunes disabled'} onClick={this.enableTunes.bind(this)}>Tunes</div>)
    let both = (<div class={this.state.includeTunes && this.state.includeMixes ? 'both enabled' : 'both disabled'} onClick={this.enableBoth.bind(this)}>Both</div>)

    let marquee = !this.state.enableVisuals ? (
      <Marquee>
        <span>Artist: {this.state.artist ? ' ' + this.state.artist : ' n/a'}</span>
        <span>Title: {this.state.title ? ' ' + this.state.title : ' n/a'}</span>
        <span>Album: {this.state.album ? ' ' + this.state.album : ' n/a'}</span>
        <span>File: {this.state.file ? ' ' + this.state.file : ' n/a'}</span>
      </Marquee>
    ) : ''

    return (
      <div className="container">
        <div className="controls">
          <div class="mixes-or-tunes">{mixes}{tunes}{both}</div>
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
        {milkDrop}
      </div>
    );
  }
}

export default HelloWorld;