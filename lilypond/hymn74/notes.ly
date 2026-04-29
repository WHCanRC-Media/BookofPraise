melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  c4 e4 e8 g4 g4 a4 a4 g4 \break

  % Line 2
  g4 c4 d4 b4 g4 g4 fis4 g4 \break

  % Line 3
  d4 f4 f8 e4 e4 g4 g8 fis4 \break

  % Line 4
  b4 g4 fis4 e4 a4 g4 fis4 e4 \break

  % Line 5
  e4 e4 e8 f4 f4 fis4 fis8 g4 \break

  % Line 6
  g4 g4 a4 g4 e4 d4 c8 c4 \bar "|."
}
