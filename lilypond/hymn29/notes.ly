melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 f4 f4 g4 a4 bes2 bes4 \break

  % Line 2
  d4 c4 bes4 g4 a4 bes2 \break

  % Line 3
  f4 f4 g4 a4 bes2 bes4 \break

  % Line 4
  d4 c4 bes4 g4 a4 bes2 \break

  % Line 5
  r4 bes4 d4 d4 c4 bes4 a2 r2 f4 \break

  % Line 6
  f4 bes4 a4 g4 g4 f2 \break

  % Line 7
  r4 f4 d4 f4 g4 f4 f2 d4 \break

  % Line 8
  d4 ees4 d4 c4 c4 \bar "|."
}
