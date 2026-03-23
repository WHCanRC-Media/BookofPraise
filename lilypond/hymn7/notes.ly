melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f4 d4 bes'4 f4 d'4 c8 bes4 f4 \break

  % Line 2
  g4 g4 f4 bes4 f4 ees4 d2 \break

  % Line 3
  f4 d4 bes'4 f4 d'4 c8 bes4 a4 \break

  % Line 4
  bes4 a4 g4 a4( bes4) a4 g4 f2 \break

  % Line 5
  c'4 c8 a4 f4 d'4 c8 bes4 g4 \break

  % Line 6
  ees'4 d4 c4 bes4 bes4 a4 bes2 \bar "|."
}
