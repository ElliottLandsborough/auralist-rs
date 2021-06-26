import React from "react";
import butterchurn from 'butterchurn';
import butterchurnPresets from 'butterchurn-presets';

export default class Milkdrop extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      isFullscreen: false,
      presets: false,
    };
    this._handleFocusedKeyboardInput = this._handleFocusedKeyboardInput.bind(
      this
    );
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

    // Kick off the animation loop
    const loop = () => {
      if (this.props.playing && this.props.isEnabledVisualizer) {
        this.visualizer.render();
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
      case 82: // R
        this.props.presets.toggleRandomize();
        break;
      case 76: // L
        this.setState({ presetOverlay: !this.state.presetOverlay });
        e.stopPropagation();
        break;
      case 145: // scroll lock
      case 125: // F14 (scroll lock for OS X)
        this.presetCycle = !this.presetCycle;
        this._restartCycling();
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
      // TODO: Kinda weird that we use the passed preset for the visualizer,
      // but set state to the current. Maybe we should just always use the curent..
      this.setState({ currentPreset: this.state.presets.getCurrentIndex() });
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
