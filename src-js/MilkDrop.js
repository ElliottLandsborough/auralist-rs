import React from "react";
import butterchurn from 'butterchurn';
import butterchurnPresets from 'butterchurn-presets';

export default class Milkdrop extends React.Component {
  constructor(props) {
    super(props);
    this.state = this.getInitialState();
  }

  getInitialState() {
    return {
      presets: [],
      preset: {
        name: '',
        item: {}
      }
    };
  }

  async componentDidMount() {
    this.setState({presets: butterchurnPresets.getPresets()});
  
    this.visualizer = butterchurn.createVisualizer(
      this.props.context,
      this._canvasNode,
      {
        width: this.props.width,
        height: this.props.height,
        meshWidth: 32,
        meshHeight: 24,
        pixelRatio: window.devicePixelRatio || 1
      }
    );

    this.visualizer.connectAudio(this.props.audio._node);
    this.visualizer.setRendererSize(this.props.width, this.props.height);
    this.loadRandomPreset();

    self = this;

    const loop = () => {
      if (self.props.playing) {
        self.visualizer.render();
      }
      this._animationFrameRequest = window.requestAnimationFrame(loop);
    };
    loop();
  }

  loadRandomPreset() {
    const preset = this.randomPreset();
    this.visualizer.loadPreset(preset.item, 2);
    this.setState({preset: preset});
  }

  randomPreset() {
    const list = butterchurnPresets.getPresets();
    const keys = Object.keys(list);
    const randomIndex = keys[Math.floor(Math.random() * keys.length)];
    const item = list[randomIndex];

    return {
      name: randomIndex,
      item: item
    };
  }

  componentWillUnmount() {
    this._pauseViz();
  }

  componentDidUpdate(prevProps) {
    if (
      this.props.width !== prevProps.width ||
      this.props.height !== prevProps.height
    ) {
      this.visualizer.setRendererSize(this.props.width, this.props.height);
    }
  }

  _pauseViz() {
    if (this._animationFrameRequest) {
      window.cancelAnimationFrame(this._animationFrameRequest);
      this._animationFrameRequest = null;
    }
  }

  handleChangeClick(e) {
    this.loadRandomPreset();
  }

  render() {
    return (
      <div className="milk-drop">
        <canvas
          height={this.props.height}
          width={this.props.width}
          ref={node => (this._canvasNode = node)}
        />
        <p>{this.state.preset.name}</p>
        <button onClick={this.handleChangeClick.bind(this)}>Change preset</button>
      </div>
    );
  }
}
