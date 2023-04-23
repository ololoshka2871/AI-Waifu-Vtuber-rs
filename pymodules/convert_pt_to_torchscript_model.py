import torch
import sys



def prepare_torch_data(self, text: str) -> dict:
    speaker = 'xenia'
    assert speaker in self.speakers, f"`speaker` should be in {', '.join(self.speakers)}"

    #import wingdbstub
    input_text = text
    speaker_ids = self.get_speakers(speaker, None)
    sentences, clean_sentences, break_lens, prosody_rates, prosody_pitches, sp_ids = \
        self.prepare_tts_model_input(input_text,
                                            ssml=None,
                                            speaker_ids=speaker_ids)
    return dict(
        sentences=sentences,
        clean_sentences=clean_sentences,
        break_lens=break_lens,
        prosody_rates=prosody_rates,
        prosody_pitches=prosody_pitches,
        speaker_ids=sp_ids,
        sr=48000,
        device=str(self.device),
        put_yo=True,
        put_accent=True
    )

pt_localfile = f'{sys.argv[1]}'
torchscript_model = f'{sys.argv[2]}' if len(sys.argv) > 2 else f'{sys.argv[1]}.torchscript.pt'

module = torch.package.PackageImporter(pt_localfile).load_pickle("tts_models", "model")

# An example input you would normally provide to your model's forward() method.
example = prepare_torch_data(module, 'Привет, мир!')

# Use torch.jit.trace to generate a torch.jit.ScriptModule via tracing.
traced_script_module = torch.jit.trace(module.model, example)

# Save the TorchScript model
traced_script_module.save(torchscript_model)