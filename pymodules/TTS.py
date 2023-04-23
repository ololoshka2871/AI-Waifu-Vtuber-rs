import io
import os
from typing import Optional
import torch
import wave


# https://github.com/snakers4/silero-models#text-to-speech
class SileroTTS:
    SAMPLE_RATE = 48000

    def __init__(self, language, model, model_file=None) -> None:
        self._localfile = f'{language}_{model}.pt' if model_file is None else model_file

        # download model if not exists
        if not os.path.isfile(self._localfile):
            torch.hub.download_url_to_file(f'https://models.silero.ai/models/tts/{language}/{model}.pt',
                                           self._localfile)

        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        torch.set_num_threads(4)

        self._model = torch.package.PackageImporter(self._localfile).load_pickle(  # type: ignore
            "tts_models", "model")
        self._model.to(self.device)

    def say_to_file(self, text: str, speaker: str, outfile=None) -> str:
        '''
        Say text using speaker and save to file
        '''
        return self._model.save_wav(text=text,
                                    speaker=speaker,
                                    audio_path=outfile)

    def _say_data(self, text: str, speaker: Optional[str]) -> bytes:
        if speaker is None:
            audiodata = SileroTTS.apply_tts(self._model, text=text)
        else:
            audiodata = SileroTTS.apply_tts(
                self._model, text=text, speaker=speaker)
        return audiodata

    def prepare_torch_data(self, text: str, speaker: Optional[str]) -> dict:
        if speaker is None:
            speaker = 'xenia'
        assert speaker in self._model.speakers, f"`speaker` should be in {', '.join(self._model.speakers)}"

        #import wingdbstub
        input_text = text
        speaker_ids = self._model.get_speakers(speaker, None)
        sentences, clean_sentences, break_lens, prosody_rates, prosody_pitches, sp_ids = \
            self._model.prepare_tts_model_input(input_text,
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

    def convert2wav(self, data_tensor) -> bytes:
        buffer = io.BytesIO()
        wf = wave.open(buffer, 'wb')
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(SileroTTS.SAMPLE_RATE)
        wf.writeframes((data_tensor * 32767).numpy().astype('int16'))
        wf.close()

        return buffer.getvalue()

    def say_wav_data(self, text: str, speaker: Optional[str] = None) -> bytes:
        '''
        Say text using speaker
        return audio_data_wav
        '''
        audiodata = self._say_data(text, speaker)

        # create audio buffer from audio_data
        buffer = io.BytesIO()
        wf = wave.open(buffer, 'wb')
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(SileroTTS.SAMPLE_RATE)
        wf.writeframes((audiodata * 32767).numpy().astype('int16'))
        wf.close()

        return buffer.getvalue()

    @staticmethod
    def apply_tts(self, text=None,
                  ssml_text=None,
                  speaker: str = 'xenia',
                  sample_rate: int = 48000,
                  put_accent=True,
                  put_yo=True,
                  voice_path=None):

        import wingdbstub

        assert sample_rate in [
            8000, 24000, 48000], f"`sample_rate` should be in [8000, 24000, 48000], current value is {sample_rate}"
        assert speaker in self.speakers, f"`speaker` should be in {', '.join(self.speakers)}"
        assert text is not None or ssml_text is not None, "Both `text` and `ssml_text` are empty"

        ssml = ssml_text is not None
        if ssml:
            input_text = ssml_text
        else:
            input_text = text
        speaker_ids = self.get_speakers(speaker, voice_path)
        sentences, clean_sentences, break_lens, prosody_rates, prosody_pitches, sp_ids = self.prepare_tts_model_input(input_text,
                                                                                                                      ssml=ssml,
                                                                                                                      speaker_ids=speaker_ids)

        with torch.no_grad():
            try:
                out, out_lens = self.model(sentences=sentences,
                                           clean_sentences=clean_sentences,
                                           break_lens=break_lens,
                                           prosody_rates=prosody_rates,
                                           prosody_pitches=prosody_pitches,
                                           speaker_ids=sp_ids,
                                           sr=sample_rate,
                                           device=str(self.device),
                                           put_yo=put_yo,
                                           put_accent=put_accent
                                           )
            except RuntimeError as e:
                raise Exception(
                    "Model couldn't generate your text, probably it's too long")
        audio = out.to('cpu')[0]
        return audio
