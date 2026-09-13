import { useState } from 'react';
import { Download, Loader2 } from 'lucide-react';
import { parseEntityRef } from '../../lib/parseResult';
import { dispatchCommand } from '../../services/commandBridge';
import { exportAudioPreset, downloadAudioResult, SFX_PRESETS, type SfxPresetName } from '../../services/audio';
import { useStudioStore } from '../../store/studioStore';
import { Panel, Button } from '../../components/ui/primitives';

export default function AudioStudio() {
  const [bpm, setBpm] = useState<number>(120);
  const [leadRow, setLeadRow] = useState<boolean[]>(Array(16).fill(false));
  const [bassRow, setBassRow] = useState<boolean[]>(Array(16).fill(false));
  const [drumRow, setDrumRow] = useState<boolean[]>(Array(16).fill(false));
  const [voice, setVoice] = useState<string>("en_US-lessac-medium");
  const [effect, setEffect] = useState<string>("none");
  const [vocalText, setVocalText] = useState<string>("");
  const [exportingPreset, setExportingPreset] = useState<SfxPresetName | null>(null);
  
  const { entities, addEntity, setLastError } = useStudioStore();

  const generateSfx = async (preset: string) => {
    try {
      const res = await dispatchCommand("generate_sfx_preset", {
        soundName: `${preset}_${entities.filter(e => e.kind === "audio_clip").length + 1}`,
        preset
      });
      const ref = parseEntityRef(res.result);
      if (ref) addEntity({ ...ref, name: preset, label: "SFX" });
    } catch (e) {
      setLastError(String(e));
    }
  };

  const exportSfx = async (preset: SfxPresetName) => {
    try {
      setExportingPreset(preset);
      const res = await exportAudioPreset(preset);
      downloadAudioResult(res);
    } catch (e) {
      setLastError(String(e));
    } finally {
      setExportingPreset(null);
    }
  };

  const generateVocal = async () => {
    try {
      const res = await dispatchCommand("synthesize_vocal", {
        clipName: `vocal_${entities.filter(e => e.kind === "audio_clip").length + 1}`,
        text: vocalText,
        voiceModel: voice,
        characterEffect: effect
      });
      const ref = parseEntityRef(res.result);
      if (ref) addEntity({ ...ref, name: "vocal", label: "Vocal" });
    } catch (e) {
      setLastError(String(e));
    }
  };

  const composeMusic = async () => {
    try {
      const res = await dispatchCommand("create_music_pattern", {
        trackName: `pattern_${entities.filter(e => e.kind === "music_track").length + 1}`,
        bpm,
        steps: 16,
        channels: [
          {
            channelIndex: 0,
            instrument: "chiptune_square",
            notes: leadRow.map((on) => (on ? "C-4" : "---"))
          },
          {
            channelIndex: 1,
            instrument: "bass",
            notes: bassRow.map((on) => (on ? "A-1" : "---"))
          },
          {
            channelIndex: 2,
            instrument: "drums",
            notes: drumRow.map((on) => (on ? "D-1" : "---"))
          }
        ]
      });
      const ref = parseEntityRef(res.result);
      if (ref) addEntity({ ...ref, name: "pattern", label: "Music Track" });
    } catch (e) {
      setLastError(String(e));
    }
  };

  return (
    <div className="flex flex-col h-full overflow-y-auto p-3 gap-3">
      <Panel title="Sound Effects (SFX)">
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
          {SFX_PRESETS.map((preset) => (
            <div key={preset} className="flex flex-col gap-1 rounded border border-border p-1.5">
              <Button size="sm" onClick={() => generateSfx(preset)}>
                {preset}
              </Button>
              <Button size="sm" variant="ghost" className="gap-1 text-[11px]" onClick={() => exportSfx(preset)} disabled={exportingPreset !== null} aria-label={`Export ${preset} WAV`}>
                {exportingPreset === preset ? <Loader2 className="h-3 w-3 animate-spin" /> : <Download className="h-3 w-3" />}
                WAV
              </Button>
            </div>
          ))}
        </div>
        <p className="mt-1 text-[11px] text-muted">Generate a preset into the scene, or export it straight to a downloadable .wav via the backend synth.</p>
      </Panel>

      <Panel title="Vocals">
        <div className="flex flex-col gap-4">
          <input
            type="text"
            placeholder="Systems operational..."
            value={vocalText}
            onChange={(e) => setVocalText(e.target.value)}
            className="rounded border border-border bg-surface px-2 py-1 text-sm"
          />

          <div className="grid grid-cols-2 gap-2">
            <select
              value={voice}
              onChange={(e) => setVoice(e.target.value)}
              className="rounded border border-border bg-surface px-2 py-1 text-sm"
            >
              <option value="en_US-lessac-medium">English (US)</option>
              <option value="en_US-amy-medium">Amy (US)</option>
              <option value="en_GB-alan-medium">Alan (UK)</option>
            </select>

            <select
              value={effect}
              onChange={(e) => setEffect(e.target.value)}
              className="rounded border border-border bg-surface px-2 py-1 text-sm"
            >
              <option value="none">None</option>
              <option value="robot">Robot</option>
              <option value="monster_growl">Monster Growl</option>
              <option value="radio_comm">Radio</option>
              <option value="alien_reverb">Alien Reverb</option>
            </select>
          </div>

          <Button onClick={generateVocal} disabled={!vocalText.trim()}>
            Generate
          </Button>
        </div>
      </Panel>

      <Panel title="Music Tracker">
        <div className="flex flex-col gap-4">
          <div className="grid grid-cols-2 gap-2">
            <input
              type="number"
              min={60}
              max={240}
              value={bpm}
              onChange={(e) => setBpm(Number(e.target.value))}
              className="rounded border border-border bg-surface px-2 py-1 text-sm"
            />
          </div>

          <div className="flex flex-col gap-2">
            <span className="text-sm">Lead</span>
            <div className="flex gap-1">
              {leadRow.map((on, i) => (
                <Button
                  key={`lead-${i}`}
                  onClick={() => {
                    const newRow = [...leadRow];
                    newRow[i] = !newRow[i];
                    setLeadRow(newRow);
                  }}
                  className={`h-6 w-6 rounded ${on ? "bg-accent" : "bg-surfaceAlt border border-border"}`}
                />
              ))}
            </div>
            <span className="text-sm">Bass</span>
            <div className="flex gap-1">
              {bassRow.map((on, i) => (
                <Button
                  key={`bass-${i}`}
                  onClick={() => {
                    const newRow = [...bassRow];
                    newRow[i] = !newRow[i];
                    setBassRow(newRow);
                  }}
                  className={`h-6 w-6 rounded ${on ? "bg-accent" : "bg-surfaceAlt border border-border"}`}
                />
              ))}
            </div>
            <span className="text-sm">Drums</span>
            <div className="flex gap-1">
              {drumRow.map((on, i) => (
                <Button
                  key={`drum-${i}`}
                  onClick={() => {
                    const newRow = [...drumRow];
                    newRow[i] = !newRow[i];
                    setDrumRow(newRow);
                  }}
                  className={`h-6 w-6 rounded ${on ? "bg-accent" : "bg-surfaceAlt border border-border"}`}
                />
              ))}
            </div>
          </div>

          <Button onClick={composeMusic}>
            Compose
          </Button>
        </div>
      </Panel>

      <Panel title="Clips">
        <div className="flex flex-col gap-1">
          {entities.filter(e => e.kind === "audio_clip" || e.kind === "music_track").map((e) => (
            <div key={e.id} className="flex justify-between text-sm">
              <span>{e.name}</span>
              <span className="text-muted">{e.kind}</span>
            </div>
          ))}
        </div>
      </Panel>
    </div>
  );
}