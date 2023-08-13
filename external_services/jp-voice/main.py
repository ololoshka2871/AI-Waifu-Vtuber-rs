#!/usr/bin/env python

import sys
import os
import io

import wave
import logging
import argparse
import functools
import pathlib

from aiohttp import web


CURRENT_PATH = pathlib.Path(__file__).parent.resolve()
SYNTHESIZER_PATH = 'voice_synthesizer_dist'

SYNTHESIZER_PATH_ABS = CURRENT_PATH.joinpath(SYNTHESIZER_PATH)

sys.path.append(str(SYNTHESIZER_PATH_ABS))

from voice_synthesizer_dist.app import models_info, create_tts_fn, create_to_symbol_fn
import voice_synthesizer_dist.utils as utils
import voice_synthesizer_dist.ONNXVITS_infer as ONNXVITS_infer


logger = logging.getLogger(__name__)


async def index(request: web.Request) -> web.Response:   
    # show api documentation
    return web.FileResponse(CURRENT_PATH.joinpath('index.html'))


async def say_get(tts: tuple, speakers: dict, request: web.Request) -> web.StreamResponse:
    params = request.rel_url.query

    # get voice_id value
    # 美妙姿势 Fine Motion (Umamusume Pretty Derby): 21
    # 美普波旁 Mihono Bourbon (Umamusume Pretty Derby): 25
    # 胜利奖券 Winning Ticket (Umamusume Pretty Derby): 34
    # 艾尼斯风神 Ines Fujin (Umamusume Pretty Derby): 30
    # 草上飞 Grass Wonder (Umamusume Pretty Derby): 10
    # 荒漠英雄 Zenno Rob Roy (Umamusume Pretty Derby): 46
    # 荣进闪耀 Eishin Flash (Umamusume Pretty Derby): 36
    # 菱亚马逊 Hishi Amazon (Umamusume Pretty Derby): 11
    # 西野花 Nishino Flower (Umamusume Pretty Derby): 50
    # 超级小溪 Super Creek (Umamusume Pretty Derby): 44
    # 醒目飞鹰 Smart Falcon (Umamusume Pretty Derby): 45
    # 采珠 Seeking the Pearl (Umamusume Pretty Derby): 41
    # 里见光钻 Satono Diamond (Umamusume Pretty Derby): 66
    # 重炮 Mayano Topgun (Umamusume Pretty Derby): 23
    # 雪之美人 Yukino Bijin (Umamusume Pretty Derby): 28
    # 青云天空 Seiun Sky (Umamusume Pretty Derby): 19
    # 青竹回忆 Bamboo Memory (Umamusume Pretty Derby): 52
    # 骏川手纲 Hayakawa Tazuna (Umamusume Pretty Derby): 81
    # 鲁道夫象征 Symboli Rudolf (Umamusume Pretty Derby): 16
    # 鹤丸刚志 Tsurumaru Tsuyoshi (Umamusume Pretty Derby): 72
    # 黄金城市 Gold City (Umamusume Pretty Derby): 39
    # 黄金船 Gold Ship (Umamusume Pretty Derby): 6
    charlist = speakers.values()
    voice_id = request.rel_url.query.get('voice_id', 6)
    if voice_id not in charlist:
        voice_id = 0

    speaker = [k for k, v in speakers.items() if v == voice_id][0]

    # get text to say
    text = params.get('text', '')

    # get duration
    # 0.1 - 5.0
    duration = max(0.1, min(5.0, float(params.get('duration', 1.0))))

    logger.info(f'Generating audio ({speaker}) for text: "{text}"')

    # generate audio file
    tts_fn = tts[6]
    _text_output, audio_output = tts_fn(text, speaker, None, duration, False)

    if _text_output == 'Success':
        # create audio buffer from audio_data
        buffer = io.BytesIO()
        wf = wave.open(buffer, 'wb')
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(audio_output[0])
        wf.writeframes((audio_output[1] * 32767).astype('int16'))
        wf.close()

        response = web.StreamResponse()
        response.headers['Content-Type'] = 'audio/wav'

        writer = await response.prepare(request)
        await writer.write(buffer.getvalue())
        await writer.drain()

        return response
    else:
        return web.Response(status=500, text="Failed to generate audio")


async def start_server() -> web.Application:
    info = models_info[1]  # Japanese model
    name = info['title']
    lang = info['languages']
    examples = info['examples']
    # patch paths with SYNTHESIZER_PATH
    config_path = str(SYNTHESIZER_PATH_ABS.joinpath(info['config_path']))
    model_path = str(SYNTHESIZER_PATH_ABS.joinpath(info['model_path']))
    description = info['description']
    onnx_dir = str(SYNTHESIZER_PATH_ABS.joinpath(info["onnx_dir"])) + "/" # костыль
    hps = utils.get_hparams_from_file(config_path)
    model = ONNXVITS_infer.SynthesizerTrn(
        len(hps.symbols),
        hps.data.filter_length // 2 + 1,
        hps.train.segment_size // hps.data.hop_length,
        n_speakers=hps.data.n_speakers,
        ONNX_dir=onnx_dir,
        **hps.model)
    utils.load_checkpoint(model_path, model, None)
    model.eval()
    speaker_ids = hps.speakers
    speakers = list(hps.speakers.keys())
    
    # models_tts = []
    # models_vc = []
    # models_tts.append((name, description, speakers, lang, examples,
    #                     hps.symbols, create_tts_fn(model, hps, speaker_ids),
    #                     create_to_symbol_fn(hps)))
    # models_vc.append((name, description, speakers, create_vc_fn(model, hps, speaker_ids)))

    models_tts = (name, description, speakers, lang, examples, 
                  hps.symbols, create_tts_fn(model, hps, speaker_ids),
                  create_to_symbol_fn(hps))

    app = web.Application()
    # call handle_request with tts as first argument
    app.add_routes([
        web.get('/', handler=index),
        web.get('/say', handler=functools.partial(say_get, models_tts, speaker_ids))
    ])
    return app


if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        description='An AI voice to text transcription server')
    
    parser.add_argument('-p', '--port', type=int,
                        default=8231, help='Port to listen on')
    args = parser.parse_args()

    logger.info(f'Starting server at http://localhost:{args.port}/')

    web.run_app(start_server(), port=args.port)
