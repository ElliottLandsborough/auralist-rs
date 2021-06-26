import React from "react";
import butterchurn from 'butterchurn';
import butterchurnPresets from 'butterchurn-presets';

export default class Milkdrop extends React.Component {
  constructor(props) {
    super(props);
  }

  async componentDidMount() {
    this.setState({presets: butterchurnPresets});
  
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

    // load a preset
    const presets = butterchurnPresets.getPresets();
    const preset = presets['Flexi, martin + geiss - dedicated to the sherwin maxawow'];

    this.visualizer.loadPreset(preset, 0.0); // 2nd argument is the number of seconds to blend presets

    this.visualizer.setRendererSize(this.props.width, this.props.height);

    self = this;

    // Kick off the animation loop
    const loop = () => {
      if (self.props.playing) {
        self.visualizer.render();
        console.log('render');
      }
      this._animationFrameRequest = window.requestAnimationFrame(loop);
    };
    loop();
  }

  componentWillUnmount() {
    this._pauseViz();
    this._stopCycling();
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

  _stopCycling() {
    if (this.cycleInterval) {
      clearInterval(this.cycleInterval);
      this.cycleInterval = null;
    }
  }

  _restartCycling() {
    this._stopCycling();

    if (this.presetCycle) {
      this.cycleInterval = setInterval(() => {
        this._nextPreset(PRESET_TRANSITION_SECONDS);
      }, MILLISECONDS_BETWEEN_PRESET_TRANSITIONS);
    }
  }

  _handleFocusedKeyboardInput(e) {
    switch (e.keyCode) {
      case 32: // spacebar
        this._nextPreset(USER_PRESET_TRANSITION_SECONDS);
        break;
      case 8: // backspace
        this._prevPreset(0);
        break;
      case 72: // H
        this._nextPreset(0);
        break;
    }
  }

  async _nextPreset(blendTime) {
    this.selectPreset(await this.state.presets.next(), blendTime);
  }

  async _prevPreset(blendTime) {
    this.selectPreset(await this.state.presets.previous(), blendTime);
  }

  selectPreset(preset, blendTime = 0) {
    if (preset != null) {
      this.visualizer.loadPreset(preset, blendTime);
      this._restartCycling();
    }
  }

  render() {
    return (
      <React.Fragment>
        <canvas
          height={this.props.height}
          width={this.props.width}
          ref={node => (this._canvasNode = node)}
        />
      </React.Fragment>
    );
  }
}
